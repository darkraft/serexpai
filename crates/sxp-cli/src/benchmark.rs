//! `benchmark OUT_DIR MANIFEST GATE`. Host is read before any arm runs.

use std::io::{Read, Write};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use sxp_bench::{Meta, SidecarChoice, benchmark_manifest};

use crate::error::CliError;
use crate::fs::{Fs, NodeKind};
use crate::install;

pub fn benchmark(
    fs: &mut dyn Fs,
    out: &Path,
    manifest: &Path,
    gate: &Path,
) -> Result<String, CliError> {
    let host = read_host()?;
    require_dir(fs, out)?;
    let manifest_bytes = read_file(fs, manifest)?;
    let gate_bytes = read_file(fs, gate)?;
    let manifest_text =
        std::str::from_utf8(&manifest_bytes).map_err(|_| CliError::token("BadManifest"))?;
    let gate_text = std::str::from_utf8(&gate_bytes).map_err(|_| CliError::token("BadManifest"))?;
    let start = unix_secs()?;
    let artifacts = benchmark_manifest(
        manifest_text,
        gate_text,
        &Meta {
            threads: 0,
            sidecar: SidecarChoice::Auto,
            host: &host,
            start: &rfc3339(start)?,
            end: &rfc3339(start)?,
            elapsed_seconds: 0,
        },
    )
    .map_err(bench_err)?;
    let end = unix_secs()?;
    if end < start {
        return Err(CliError::token("Io"));
    }
    let timing = format!(
        "host={host}\nscope=benchmark\nstart={}\nend={}\ncompleted=true\nbudget_seconds=86400\nwithin_budget={}\n",
        rfc3339(start)?,
        rfc3339(end)?,
        if end.saturating_sub(start) <= 86_400 {
            "true"
        } else {
            "false"
        }
    );
    for (name, bytes) in &artifacts.sidecars {
        store(fs, out, name, bytes)?;
    }
    store(fs, out, "annex.json", artifacts.annex.as_bytes())?;
    store(fs, out, "timing.log", timing.as_bytes())?;
    store(fs, out, "report.json", artifacts.report.as_bytes())?;
    Ok(artifacts.report)
}

fn bench_err(err: sxp_bench::Error) -> CliError {
    CliError::token_owned(err.to_string())
}

fn read_host() -> Result<String, CliError> {
    let text = std::fs::read_to_string("/etc/hostname").map_err(|err| CliError::from_io(&err))?;
    let host = text.trim();
    if host.is_empty() || host.chars().any(|c| c.is_control() || c == '=') {
        return Err(CliError::token("Io"));
    }
    Ok(host.to_string())
}

fn require_dir(fs: &dyn Fs, path: &Path) -> Result<(), CliError> {
    match fs.symlink_meta(path) {
        Ok(meta) if meta.kind == NodeKind::Dir => Ok(()),
        Ok(_) => Err(CliError::token("NotADirectory")),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Err(CliError::token("NotFound")),
        Err(err) => Err(CliError::from_io(&err)),
    }
}

fn read_file(fs: &mut dyn Fs, path: &Path) -> Result<Vec<u8>, CliError> {
    match fs.meta(path) {
        Ok(meta) if meta.kind == NodeKind::Dir => return Err(CliError::token("IsADirectory")),
        Ok(meta) if meta.kind != NodeKind::File => return Err(CliError::token("Unsupported")),
        Ok(_) => {}
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            return Err(CliError::token("NotFound"));
        }
        Err(err) => return Err(CliError::from_io(&err)),
    }
    let mut reader = fs.open_read(path).map_err(|err| CliError::from_io(&err))?;
    let mut buf = Vec::new();
    let mut chunk = [0u8; 8192];
    loop {
        let n = match reader.read(&mut chunk) {
            Ok(n) => n,
            Err(err) if err.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(err) => return Err(CliError::from_io(&err)),
        };
        if n == 0 {
            break;
        }
        if buf.len().saturating_add(n) > 8 * 1024 * 1024 {
            return Err(CliError::token("BadManifest"));
        }
        let got = chunk.get(..n).ok_or_else(|| CliError::token("Io"))?;
        buf.extend_from_slice(got);
    }
    Ok(buf)
}

fn store(fs: &mut dyn Fs, dir: &Path, name: &str, bytes: &[u8]) -> Result<(), CliError> {
    if name.is_empty() || name.contains('/') || name.contains("..") {
        return Err(CliError::token("BadManifest"));
    }
    let dst = dir.join(name);
    let (temp, mut file) = install::create_temp(fs, &dst)?;
    let wrote = file
        .write_all(bytes)
        .and_then(|()| file.flush())
        .and_then(|()| file.sync_all());
    drop(file);
    if let Err(err) = wrote {
        install::abandon(fs, &temp);
        return Err(CliError::from_io(&err));
    }
    if let Err(err) = fs.rename(&temp, &dst) {
        install::abandon(fs, &temp);
        return Err(CliError::from_io(&err));
    }
    Ok(())
}

fn unix_secs() -> Result<u64, CliError> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .map_err(|_| CliError::token("Io"))
}

fn rfc3339(secs: u64) -> Result<String, CliError> {
    let days = i64::try_from(secs / 86_400).map_err(|_| CliError::token("Io"))?;
    let rem = secs % 86_400;
    let hour = rem / 3_600;
    let minute = (rem % 3_600) / 60;
    let second = rem % 60;
    let (year, month, day) = civil(days)?;
    Ok(format!(
        "{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}Z"
    ))
}

fn civil(z: i64) -> Result<(i64, u32, u32), CliError> {
    let z = z
        .checked_add(719_468)
        .ok_or_else(|| CliError::token("Io"))?;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = u64::try_from(z - era * 146_097).map_err(|_| CliError::token("Io"))?;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let mut year = i64::from(u32::try_from(yoe).map_err(|_| CliError::token("Io"))?)
        .checked_add(era.checked_mul(400).ok_or_else(|| CliError::token("Io"))?)
        .ok_or_else(|| CliError::token("Io"))?;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = u32::try_from(doy - (153 * mp + 2) / 5 + 1).map_err(|_| CliError::token("Io"))?;
    let month =
        u32::try_from(if mp < 10 { mp + 3 } else { mp - 9 }).map_err(|_| CliError::token("Io"))?;
    if month <= 2 {
        year = year.checked_add(1).ok_or_else(|| CliError::token("Io"))?;
    }
    if !(0..=9999).contains(&year) {
        return Err(CliError::token("Io"));
    }
    Ok((year, month, day))
}
