use systemstat::{
    Platform, System,
    data::{ByteSize, saturating_sub_bytes},
};

use super::{Context, Module, ModuleConfig};

use crate::configs::memory_usage::MemoryConfig;
use crate::formatter::StringFormatter;

// Display a `ByteSize` in a specific unit chosen by the user
fn display_bs_with_unit(bs: ByteSize, target_unit: &str) -> String {
    match target_unit {
        "B" => format!("{}B", bs.0),

        "KiB" => format!("{:.1}KiB", bs.0 as f64 / 1024.0),
        "MiB" => format!("{:.1}MiB", bs.0 as f64 / (1024.0 * 1024.0)),
        "GiB" => format!("{:.1}GiB", bs.0 as f64 / (1024.0 * 1024.0 * 1024.0)),
        "TiB" => format!("{:.1}TiB", bs.0 as f64 / (1024.0 * 1024.0 * 1024.0 * 1024.0)),

        "KB" => format!("{:.1}KB", bs.0 as f64 / 1000.0),
        "MB" => format!("{:.1}MB", bs.0 as f64 / (1000.0 * 1000.0)),
        "GB" => format!("{:.1}GB", bs.0 as f64 / (1000.0 * 1000.0 * 1000.0)),
        "TB" => format!("{:.1}TB", bs.0 as f64 / (1000.0 * 1000.0 * 1000.0 * 1000.0)),

        "K" => format!("{:.1}K", bs.0 as f64 / 1000.0),
        "M" => format!("{:.1}M", bs.0 as f64 / (1000.0 * 1000.0)),
        "G" => format!("{:.1}G", bs.0 as f64 / (1000.0 * 1000.0 * 1000.0)),
        "T" => format!("{:.1}T", bs.0 as f64 / (1000.0 * 1000.0 * 1000.0 * 1000.0)),

        "auto" => display_bs_auto(bs), // comportamento automatico originale
        _ => display_bs_auto(bs), // fallback al comportamento automatico
    }
}

// Display a `ByteSize` in a human readable format (comportamento originale).
fn display_bs_auto(bs: ByteSize) -> String {
    let mut display_bytes = bs.to_string_as(true);
    let mut keep = true;
    // Skip decimals and the space before the byte unit.
    display_bytes.retain(|c| match c {
        ' ' => {
            keep = true;
            false
        }
        '.' => {
            keep = false;
            false
        }
        _ => keep,
    });
    display_bytes
}

// Calculate the memory usage from total and free memory
fn pct(total: ByteSize, free: ByteSize) -> f64 {
    100.0 * saturating_sub_bytes(total, free).0 as f64 / total.0 as f64
}

// Print usage string used/total with custom unit
fn format_usage_total_with_unit(total: ByteSize, free: ByteSize, unit: &str) -> String {
    format!(
        "{}/{}",
        display_bs_with_unit(saturating_sub_bytes(total, free), unit),
        display_bs_with_unit(total, unit)
    )
}

/// Creates a module with system memory usage information
pub fn module<'a>(context: &'a Context) -> Option<Module<'a>> {
    let mut module = context.new_module("memory_usage");
    let config = MemoryConfig::try_load(module.config);

    // As we default to disabled=true, we have to check here after loading our config module,
    // before it was only checking against whatever is in the config starship.toml
    if config.disabled {
        return None;
    }

    let system = System::new();

    // `memory_and_swap` only works on platforms that have an implementation for swap memory
    // But getting both together is faster on some platforms (Windows/Linux)
    let (memory, swap) = match system.memory_and_swap() {
        // Ignore swap if total is 0
        Ok((mem, swap)) if swap.total.0 > 0 => (mem, Some(swap)),
        Ok((mem, _)) => (mem, None),
        Err(e) => {
            log::debug!(
                "Failed to retrieve both memory and swap, falling back to memory only: {e}"
            );
            let mem = match system.memory() {
                Ok(mem) => mem,
                Err(e) => {
                    log::warn!("Failed to retrieve memory: {e}");
                    return None;
                }
            };

            (mem, None)
        }
    };

    let used_pct = pct(memory.total, memory.free);
    let ram_used = saturating_sub_bytes(memory.total, memory.free);

    if (used_pct.round() as i64) < config.threshold {
        return None;
    }

    let parsed = StringFormatter::new(config.format).and_then(|formatter| {
        formatter
            .map_meta(|var, _| match var {
                "symbol" => Some(config.symbol),
                _ => None,
            })
            .map_style(|variable| match variable {
                "style" => Some(Ok(config.style)),
                _ => None,
            })
            .map(|variable| match variable {
                "ram" => Some(Ok(format_usage_total_with_unit(memory.total, memory.free, &config.unit))),
                "ram_pct" => Some(Ok(format!("{used_pct:.0}%"))),
                "used" => Some(Ok(display_bs_with_unit(ram_used, &config.unit))), // Usa l'unità personalizzata
                "swap" => Some(Ok(format_usage_total_with_unit(
                    swap.as_ref()?.total,
                    swap.as_ref()?.free,
                    &config.unit,
                ))),
                "swap_pct" => Some(Ok(format!(
                    "{:.0}%",
                    pct(swap.as_ref()?.total, swap.as_ref()?.free)
                ))),
                _ => None,
            })
            .parse(None, Some(context))
    });

    module.set_segments(match parsed {
        Ok(segments) => segments,
        Err(error) => {
            log::warn!("Error in module `memory_usage`:\n{error}");
            return None;
        }
    });

    Some(module)
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::test::ModuleRenderer;

    #[test]
    fn test_format_usage_total_with_unit_gib() {
        assert_eq!(
            format_usage_total_with_unit(ByteSize(1024 * 1024 * 1024), ByteSize(1024 * 1024 * 1024), "GiB"),
            "0.0GiB/1.0GiB"
        );
        assert_eq!(
            format_usage_total_with_unit(
                ByteSize(1024 * 1024 * 1024),
                ByteSize(1024 * 1024 * 1024 / 2),
                "GiB"
            ),
            "0.5GiB/1.0GiB"
        );
        assert_eq!(
            format_usage_total_with_unit(ByteSize(1024 * 1024 * 1024), ByteSize(0), "GiB"),
            "1.0GiB/1.0GiB"
        );
    }

    #[test]
    fn test_format_usage_total_with_unit_mb() {
        assert_eq!(
            format_usage_total_with_unit(ByteSize(1000 * 1000), ByteSize(500 * 1000), "MB"),
            "0.5MB/1.0MB"
        );
    }

    #[test]
    fn test_display_bs_with_unit() {
        assert_eq!(display_bs_with_unit(ByteSize(1024), "KiB"), "1.0KiB");
        assert_eq!(display_bs_with_unit(ByteSize(1024 * 1024), "MiB"), "1.0MiB");
        assert_eq!(display_bs_with_unit(ByteSize(1000 * 1000), "MB"), "1.0MB");
        assert_eq!(display_bs_with_unit(ByteSize(1024), "B"), "1024B");
    }

    #[test]
    fn test_pct() {
        assert_eq!(
            pct(ByteSize(1024 * 1024 * 1024), ByteSize(1024 * 1024 * 1024)),
            0.0
        );
        assert_eq!(
            pct(
                ByteSize(1024 * 1024 * 1024),
                ByteSize(1024 * 1024 * 1024 / 2)
            ),
            50.0
        );
        assert_eq!(pct(ByteSize(1024 * 1024 * 1024), ByteSize(0)), 100.0);
    }

    #[test]
    fn zero_threshold() {
        let output = ModuleRenderer::new("memory_usage")
            .config(toml::toml! {
                [memory_usage]
                disabled = false
                threshold = 0
                unit = "auto"
            })
            .collect();

        assert!(output.is_some())
    }

    #[test]
    fn impossible_threshold() {
        let output = ModuleRenderer::new("memory_usage")
            .config(toml::toml! {
                [memory_usage]
                disabled = false
                threshold = 9999
                unit = "auto"
            })
            .collect();

        assert!(output.is_none())
    }

    #[test]
    fn custom_unit_gib() {
        let output = ModuleRenderer::new("memory_usage")
            .config(toml::toml! {
                [memory_usage]
                disabled = false
                threshold = 0
                unit = "GiB"
            })
            .collect();

        assert!(output.is_some())
    }
}