# Demo & notes in Rust 🦀

```bash
cargo run
```

Exploiting the Vulnerability
The attack vector is identical. Open a new terminal and use curl.

1. Request a Legitimate File:

Bash

curl http://127.0.0.1:8080/static/index.html

2.The Attack:
Now, let's craft our malicious URL. The path structure is the same.

Bash

curl http://127.0.0.1:8080/static/../../etc/passwd

Boom! The server again responds with the contents of our simulated /etc/passwd file:

root:x:0:0:root:/root:/bin/bash

----
Directory Traversal Under the Hood: The Vulnerable Code
The vulnerability exists because we blindly trust user input. The change to using App::service() doesn't alter the core problem. The handler function is what matters:

Rust

// 1. The filename is taken directly from the URL.
let filename: String = req.match_info().get("filename").unwrap().to_string();

// 2. The untrusted `filename` is joined with the base directory.
//    PathBuf::join has no concept of security boundaries. It just
//    follows filesystem rules. `public/` + `../../etc/passwd` becomes
//    `public/../../etc/passwd`, which the OS resolves correctly.
let file_path = PathBuf::from("./public/").join(&filename);
The root cause is unchanged: the application forwards unvalidated input from the web request directly to a filesystem API.
---

## The Fix

```bash
async fn serve_static_secure(req: HttpRequest) -> Result<HttpResponse, Error> {...} # see actual impl in main.rs
```

How This Mitigation Works in Rust:
fs::canonicalize: This function is the cornerstone of the fix. It asks the operating system to resolve a given path into its absolute, final form. For example, a malicious path like ./public/../../etc/passwd is resolved to its true location, such as /home/user/my_app/etc/passwd.
.starts_with() Check: By comparing the canonical path of the requested file with the canonical path of the base directory, we can definitively tell if the file is located within the allowed folder. The starts_with method on Path provides a reliable, string-based check for this parent-child relationship between the two absolute paths. If the check fails, we know a traversal was attempted, and we can deny the request.

Don't Reinvent the Wheel: Use Open-Source Crates
