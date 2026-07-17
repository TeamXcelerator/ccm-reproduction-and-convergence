# Private-cache practice run

The v0.13.0 practice path uses the private topology registry and its family shards. A GitHub PAT is stored only through Git Credential Manager; it is never placed in this repository, a request document, a command-line argument, or an environment variable.

From Windows PowerShell, provision or verify the credential with:

```powershell
.\scripts\setup_private_cache_auth.ps1
```

Use `-ReplaceCredential` to replace an existing GitHub credential. The script performs read-only live permission probes for the private registry and all seven CCM-related private shards. It does not run the paper computation and does not mutate GitHub.

The eventual practice run will use `XC_CACHE_MODE=fetch`, an explicit private publication target, owner-direct authority, registry-routed destinations, and `execute_remote_mutations=true`. Those controls will be exposed together by the run wrapper; possession of a PAT alone will never enable publication.

Do not manually upload the legacy files under `data/*_cache`. They must first be converted into v0.13.0 canonical manifests, validated, deterministically packaged, and journaled through the toolkit publication transaction. The run wrapper must not be enabled until that hook is complete.
