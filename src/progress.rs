use anyhow::Result;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::time::Duration;

pub fn progress_style_known() -> Result<ProgressStyle> {
    Ok(ProgressStyle::with_template(
        "{msg:8} {bar:30.cyan/blue} {percent:>3}% {bytes}/{total_bytes} {bytes_per_sec} ETA {eta}",
    )?
    .progress_chars("=>-"))
}

pub fn progress_style_unknown() -> Result<ProgressStyle> {
    Ok(ProgressStyle::with_template(
        "{msg:8} {spinner:.cyan} {bytes} {bytes_per_sec}",
    )?)
}

pub fn make_download_progress_bar(
    multi: Option<&MultiProgress>,
    label: &str,
    quiet: bool,
) -> Result<ProgressBar> {
    if quiet {
        return Ok(ProgressBar::hidden());
    }

    let pb = ProgressBar::new_spinner();
    pb.enable_steady_tick(Duration::from_millis(120));
    pb.set_style(progress_style_unknown()?);
    pb.set_message(label.to_string());

    Ok(match multi {
        Some(m) => m.add(pb),
        None => pb,
    })
}

pub fn make_phase_spinner(quiet: bool) -> Result<ProgressBar> {
    if quiet {
        return Ok(ProgressBar::hidden());
    }

    let spinner = ProgressBar::new_spinner();
    spinner.set_style(ProgressStyle::with_template("{spinner:.green} {msg}")?);
    spinner.enable_steady_tick(Duration::from_millis(120));
    Ok(spinner)
}
