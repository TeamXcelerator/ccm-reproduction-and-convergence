//! Own a preparer's entire Windows process tree, including on timeout or unwind.
use std::{
    io,
    mem::{size_of, zeroed},
    os::windows::{
        io::{AsRawHandle, FromRawHandle, OwnedHandle},
        process::CommandExt,
    },
    process::{Child, Command},
};
use windows_sys::Win32::{
    Foundation::INVALID_HANDLE_VALUE,
    System::{
        Diagnostics::ToolHelp::{
            CreateToolhelp32Snapshot, Thread32First, Thread32Next, TH32CS_SNAPTHREAD, THREADENTRY32,
        },
        JobObjects::{
            AssignProcessToJobObject, CreateJobObjectW, JobObjectExtendedLimitInformation,
            SetInformationJobObject, JOBOBJECT_EXTENDED_LIMIT_INFORMATION,
            JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
        },
        Threading::{
            OpenThread, ResumeThread, CREATE_NO_WINDOW, CREATE_SUSPENDED, THREAD_SUSPEND_RESUME,
        },
    },
};

pub struct Job(OwnedHandle);

/// Spawn suspended, assign ownership, then resume. No user code can run before
/// job assignment, so even an immediately spawning child cannot escape cleanup.
pub fn spawn(command: &mut Command) -> io::Result<(Child, Job)> {
    let raw = unsafe { CreateJobObjectW(std::ptr::null(), std::ptr::null()) };
    if raw.is_null() {
        return Err(io::Error::last_os_error());
    }
    let job = Job(unsafe { OwnedHandle::from_raw_handle(raw) });
    let mut limits: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = unsafe { zeroed() };
    limits.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
    if unsafe {
        SetInformationJobObject(
            raw,
            JobObjectExtendedLimitInformation,
            (&limits as *const JOBOBJECT_EXTENDED_LIMIT_INFORMATION).cast(),
            size_of_val(&limits) as u32,
        )
    } == 0
    {
        return Err(io::Error::last_os_error());
    }
    let mut child = command
        .creation_flags(CREATE_NO_WINDOW | CREATE_SUSPENDED)
        .spawn()?;
    let result = (|| {
        if unsafe { AssignProcessToJobObject(job.0.as_raw_handle(), child.as_raw_handle()) } == 0 {
            return Err(io::Error::last_os_error());
        }
        resume_initial_thread(child.id())
    })();
    if let Err(error) = result {
        let _ = child.kill();
        let _ = child.wait();
        return Err(error);
    }
    Ok((child, job))
}

fn resume_initial_thread(pid: u32) -> io::Result<()> {
    let raw = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, 0) };
    if raw == INVALID_HANDLE_VALUE {
        return Err(io::Error::last_os_error());
    }
    let snapshot = unsafe { OwnedHandle::from_raw_handle(raw) };
    let mut entry: THREADENTRY32 = unsafe { zeroed() };
    entry.dwSize = size_of::<THREADENTRY32>() as u32;
    let mut valid = unsafe { Thread32First(snapshot.as_raw_handle(), &mut entry) };
    while valid != 0 {
        if entry.th32OwnerProcessID == pid {
            let raw = unsafe { OpenThread(THREAD_SUSPEND_RESUME, 0, entry.th32ThreadID) };
            if raw.is_null() {
                return Err(io::Error::last_os_error());
            }
            let thread = unsafe { OwnedHandle::from_raw_handle(raw) };
            if unsafe { ResumeThread(thread.as_raw_handle()) } == u32::MAX {
                return Err(io::Error::last_os_error());
            }
            return Ok(());
        }
        valid = unsafe { Thread32Next(snapshot.as_raw_handle(), &mut entry) };
    }
    Err(io::Error::new(
        io::ErrorKind::NotFound,
        "suspended preparer thread unavailable",
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        fs,
        time::{Duration, Instant},
    };
    use windows_sys::Win32::System::Threading::{
        OpenProcess, WaitForSingleObject, PROCESS_SYNCHRONIZE,
    };

    #[test]
    fn owned_process_can_complete() {
        let (mut child, _job) = spawn(Command::new("cmd.exe").args(["/C", "exit 0"])).unwrap();
        assert!(child.wait().unwrap().success());
    }

    #[test]
    fn dropping_job_terminates_descendants() {
        let path = std::env::temp_dir().join(format!("paper1-child-{}.txt", std::process::id()));
        let _ = fs::remove_file(&path);
        let (mut child, job) = spawn(
            Command::new(std::env::current_exe().unwrap())
                .args([
                    "--exact",
                    "windows_process::tests::descendant_entry",
                    "--ignored",
                ])
                .env("PAPER1_TEST_CHILD_PID", &path),
        )
        .unwrap();
        let started = Instant::now();
        while !path.exists() && started.elapsed() < Duration::from_secs(10) {
            std::thread::sleep(Duration::from_millis(10));
        }
        let pid = fs::read_to_string(&path).unwrap().parse::<u32>().unwrap();
        let raw = unsafe { OpenProcess(PROCESS_SYNCHRONIZE, 0, pid) };
        assert!(!raw.is_null());
        let descendant = unsafe { OwnedHandle::from_raw_handle(raw) };
        assert!(child.try_wait().unwrap().is_none());
        assert_eq!(
            unsafe { WaitForSingleObject(descendant.as_raw_handle(), 0) },
            258
        );
        drop(job);
        child.wait().unwrap(); // Kill-on-close does not promise a nonzero exit code.
        assert_eq!(
            unsafe { WaitForSingleObject(descendant.as_raw_handle(), 10000) },
            0
        );
        fs::remove_file(path).unwrap();
    }

    #[test]
    #[ignore = "subprocess fixture, invoked by dropping_job_terminates_descendants"]
    fn descendant_entry() {
        let Some(path) = std::env::var_os("PAPER1_TEST_CHILD_PID") else {
            return;
        };
        let mut child = Command::new("cmd.exe")
            .args(["/C", "ping -n 60 127.0.0.1 >nul"])
            .creation_flags(CREATE_NO_WINDOW)
            .spawn()
            .unwrap();
        fs::write(path, child.id().to_string()).unwrap();
        let _ = child.wait();
    }
}
