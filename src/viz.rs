use std::path::Path;

/// Visualize a CLOS topology as a PNG.
///
/// When the `viz` Cargo feature is enabled, this uses the pure-Rust `plotters` crate.
/// Otherwise it prints a message telling the user to use the sibling Python project
/// (which uses matplotlib and produces higher quality diagrams matching the original).
pub fn visualize_clos(links: &[[u32; 3]], output: &Path, title: &str) -> Result<(), String> {
    #[cfg(feature = "viz")]
    {
        visualize_clos_plotters(links, output, title)
    }

    #[cfg(not(feature = "viz"))]
    {
        println!(
            "(viz feature not enabled) For high-quality diagrams, run:\n  \
             cd ../clos-dragonfly-simulation && uv run clos-visualize {} --output {}",
            output.display(),
            output.display()
        );
        // We still succeed so that generate/sweep don't fail.
        Ok(())
    }
}

/// Visualize a Dragonfly topology as a PNG.
pub fn visualize_dragonfly(
    links: &[[u32; 3]],
    output: &Path,
    title: &str,
    num_hosts: u32,
    a: u32,
    g: u32,
) -> Result<(), String> {
    #[cfg(feature = "viz")]
    {
        visualize_dragonfly_plotters(links, output, title, num_hosts, a, g)
    }

    #[cfg(not(feature = "viz"))]
    {
        println!(
            "(viz feature not enabled) For high-quality diagrams, run:\n  \
             cd ../clos-dragonfly-simulation && uv run dragonfly-visualize {} --output {}",
            output.display(),
            output.display()
        );
        Ok(())
    }
}

// -----------------------------------------------------------------------------
// Pure-Rust implementation using plotters (only compiled with --features viz)
// -----------------------------------------------------------------------------

#[cfg(feature = "viz")]
use plotters::prelude::*;
#[cfg(feature = "viz")]
use plotters::series::{DashedLineSeries, LineSeries};
#[cfg(feature = "viz")]
use plotters::style::{RGBColor, ShapeStyle};
#[cfg(feature = "viz")]
use plotters::element::Text;

#[cfg(feature = "viz")]
fn visualize_clos_plotters(links: &[[u32; 3]], output: &Path, title: &str) -> Result<(), String> {
    // Parse like the original Python code
    let mut all_ids: std::collections::HashSet<u32> = std::collections::HashSet::new();
    for &[src, dst, _] in links {
        all_ids.insert(src);
        all_ids.insert(dst);
    }

    let src_set: std::collections::HashSet<u32> = links.iter().map(|&[s, _, _]| s).collect();
    let dst_set: std::collections::HashSet<u32> = links.iter().map(|&[_, d, _]| d).collect();

    let mut hosts: Vec<u32> = src_set.difference(&dst_set).cloned().collect();
    let mut spines: Vec<u32> = dst_set.difference(&src_set).cloned().collect();
    let mut leafs: Vec<u32> = all_ids
        .difference(&src_set)
        .chain(all_ids.difference(&dst_set))
        .cloned()
        .collect();

    hosts.sort();
    leafs.sort();
    spines.sort();

    let n = hosts.len().max(leafs.len()).max(spines.len());
    // Dynamic scaling + aspect ratio (clamped for stability)
    let base_width: u32 = 900;
    let base_height: u32 = 550;
    let scale_factor = ((n as f32 / 10.0).max(0.8)).min(5.0);
    let width: u32 = (base_width as f32 * scale_factor) as u32;
    let height: u32 = (base_height as f32 * scale_factor) as u32;

    let mut buffer = vec![0u8; (width * height * 3) as usize];
    {
        let root = BitMapBackend::with_buffer(&mut buffer, (width, height)).into_drawing_area();
        root.fill(&WHITE)
            .map_err(|e| format!("failed to fill: {e}"))?;

        let mut chart = ChartBuilder::on(&root)
            .margin(30)
            .build_cartesian_2d(0f32..100f32, 0f32..100f32)
            .map_err(|e| e.to_string())?;

        // Use same Y layers as Python, scaled
        let y_host = 15.0;
        let y_leaf = 50.0;
        let y_spine = 85.0;

        let max_layer_width = [hosts.len(), leafs.len(), spines.len()]
            .iter()
            .max()
            .copied()
            .unwrap_or(1) as f32;
        let x_span = (max_layer_width * 0.5).max(4.0);

        let positions = |ids: &[u32], y: f32| -> std::collections::HashMap<u32, (f32, f32)> {
            let n = ids.len();
            if n == 0 {
                return std::collections::HashMap::new();
            }
            if n == 1 {
                let mut m = std::collections::HashMap::new();
                m.insert(ids[0], (x_span / 2.0, y));
                return m;
            }
            let spacing = x_span / (n as f32 - 1.0);
            ids.iter()
                .enumerate()
                .map(|(i, &id)| (id, (i as f32 * spacing + 10.0, y)))
                .collect()
        };

        let host_pos = positions(&hosts, y_host);
        let leaf_pos = positions(&leafs, y_leaf);
        let spine_pos = positions(&spines, y_spine);

        // Draw links with colors from Python
        for &[src, dst, _] in links {
            let p1 = host_pos
                .get(&src)
                .or_else(|| leaf_pos.get(&src))
                .or_else(|| spine_pos.get(&src));
            let p2 = host_pos
                .get(&dst)
                .or_else(|| leaf_pos.get(&dst))
                .or_else(|| spine_pos.get(&dst));

            if let (Some(&(x1, y1)), Some(&(x2, y2))) = (p1, p2) {
                let color = if hosts.contains(&src) {
                    &RGBColor(139, 187, 224) // host_link
                } else {
                    &RGBColor(196, 196, 196) // uplink
                };
                chart
                    .draw_series(LineSeries::new(vec![(x1, y1), (x2, y2)], color.stroke_width(2)))
                    .map_err(|e| e.to_string())?;
            }
        }

        // Draw nodes with colors and shapes from Python + labels when small
        let label_small = hosts.len() <= 32;
        for (i, &id) in hosts.iter().enumerate() {
            let x = 10.0 + i as f32 * (x_span / hosts.len().max(1) as f32);
            chart
                .draw_series(std::iter::once(Rectangle::new(
                    [(x - 2.5, y_host - 2.5), (x + 2.5, y_host + 2.5)],
                    &RGBColor(74, 144, 217), // host
                )))
                .map_err(|e| e.to_string())?;
            // Label for small N (requires font feature in plotters for full support)
            // if label_small { ... Text ... }
        }
        for (i, &id) in leafs.iter().enumerate() {
            let x = 10.0 + i as f32 * (x_span / leafs.len().max(1) as f32);
            chart
                .draw_series(std::iter::once(Circle::new(
                    (x, y_leaf),
                    5,
                    &RGBColor(232, 145, 58), // leaf
                )))
                .map_err(|e| e.to_string())?;
            // Label for small N (requires font feature in plotters for full support)
            // if label_small { ... }
        }
        for (i, &id) in spines.iter().enumerate() {
            let x = 10.0 + i as f32 * (x_span / spines.len().max(1) as f32);
            // Use a diamond-like shape with two triangles or just larger circle for simplicity
            chart
                .draw_series(std::iter::once(Circle::new(
                    (x, y_spine),
                    6,
                    &RGBColor(80, 184, 108), // spine
                )))
                .map_err(|e| e.to_string())?;
            // Label for small N (requires font feature in plotters for full support)
            // if label_small { ... }
        }

        // Layer annotations (text requires extra font support in plotters; shapes + colors used instead)
        // Legend via series if available
        let _ = chart
            .configure_series_labels()
            .border_style(&BLACK)
            .background_style(&WHITE.mix(0.8))
            .draw();

        root.present()
            .map_err(|e| format!("present error: {e}"))?;
    }

    use image::{ImageBuffer, Rgb};
    let img: ImageBuffer<Rgb<u8>, _> =
        ImageBuffer::from_raw(width, height, buffer).ok_or("failed to create image buffer")?;
    img.save(output)
        .map_err(|e| format!("failed to save png: {e}"))?;

    Ok(())
}

#[cfg(feature = "viz")]
fn visualize_dragonfly_plotters(
    links: &[[u32; 3]],
    output: &Path,
    title: &str,
    num_hosts: u32,
    a: u32,
    g: u32,
) -> Result<(), String> {
    // Parse like original
    let mut all_ids: std::collections::HashSet<u32> = std::collections::HashSet::new();
    for &[src, dst, _] in links {
        all_ids.insert(src);
        all_ids.insert(dst);
    }
    let src_set: std::collections::HashSet<u32> = links.iter().map(|&[s, _, _]| s).collect();
    let dst_set: std::collections::HashSet<u32> = links.iter().map(|&[_, d, _]| d).collect();

    let hosts: Vec<u32> = src_set.difference(&dst_set).cloned().collect();
    let routers: Vec<u32> = all_ids.difference(&hosts.iter().cloned().collect()).cloned().collect();

    // Dynamic scaling
    let n = (hosts.len() + routers.len()).max(1) as f32;
    let scale = (n / 20.0).max(1.0).min(4.0);
    let width: u32 = (900.0 * scale) as u32;
    let height: u32 = (700.0 * scale) as u32;

    let mut buffer = vec![0u8; (width * height * 3) as usize];
    {
        let root = BitMapBackend::with_buffer(&mut buffer, (width, height)).into_drawing_area();
        root.fill(&WHITE)
            .map_err(|e| format!("bitmap error: {e}"))?;

        let mut chart = ChartBuilder::on(&root)
            .margin(20)
            .build_cartesian_2d(0f32..100f32, 0f32..100f32)
            .map_err(|e| e.to_string())?;

        // More accurate circular groups + intra-group placement
        let cx = 50f32;
        let cy = 45f32;
        let r = 30f32;

        let group_centers: Vec<(f32, f32)> = if g > 0 {
            (0..g)
                .map(|gi| {
                    let ang = 2.0 * std::f32::consts::PI * gi as f32 / g as f32;
                    (cx + r * ang.cos(), cy + r * ang.sin() * 0.9)
                })
                .collect()
        } else {
            vec![(50.0, 50.0)]
        };

        // Build router positions inside groups for better placement
        let mut router_pos: std::collections::HashMap<u32, (f32, f32)> = std::collections::HashMap::new();
        if a > 0 && g > 0 {
            let router_start = num_hosts;
            for (gi, &(gx, gy)) in group_centers.iter().enumerate() {
                for ri in 0..a {
                    let r_idx = gi * a as usize + ri as usize;
                    let rid = router_start + r_idx as u32;
                    // Small circle or grid inside group
                    let sub_ang = 2.0 * std::f32::consts::PI * ri as f32 / a as f32;
                    let sub_r = 5.0;
                    let rx = gx + sub_r * sub_ang.cos();
                    let ry = gy + sub_r * sub_ang.sin() * 0.7;
                    router_pos.insert(rid, (rx, ry));
                }
            }
        }

        // Classify and draw links with different styles
        let router_set: std::collections::HashSet<u32> = routers.iter().cloned().collect();
        for &[s, d, _] in links {
            let p1 = if hosts.contains(&s) {
                // host position approx below router
                if let Some(& (rx, ry)) = router_pos.get(&d) {
                    Some((rx + ((s as f32 % 5.0) - 2.0) * 1.2, ry - 8.0))
                } else {
                    None
                }
            } else {
                router_pos.get(&s).copied()
            };
            let p2 = if hosts.contains(&d) {
                if let Some(& (rx, ry)) = router_pos.get(&s) {
                    Some((rx + ((d as f32 % 5.0) - 2.0) * 1.2, ry - 8.0))
                } else {
                    None
                }
            } else {
                router_pos.get(&d).copied()
            };

            if let (Some((x1, y1)), Some((x2, y2))) = (p1, p2) {
                let is_host = hosts.contains(&s) || hosts.contains(&d);
                let is_local = !is_host && {
                    let r1 = s.max(d) - num_hosts;
                    let r2 = s.min(d) - num_hosts;
                    (r1 / a) == (r2 / a)
                };

                if is_host {
                    chart
                        .draw_series(LineSeries::new(vec![(x1, y1), (x2, y2)], RGBColor(139, 187, 224).stroke_width(2)))
                        .map_err(|e| e.to_string())?;
                } else if is_local {
                    chart
                        .draw_series(LineSeries::new(vec![(x1, y1), (x2, y2)], RGBColor(232, 145, 58).stroke_width(2)))
                        .map_err(|e| e.to_string())?;
                } else {
                    // Global: dashed
                    let style = ShapeStyle {
                        color: RGBColor(153, 153, 153).to_rgba(),
                        filled: false,
                        stroke_width: 2,
                    };
                    chart
                        .draw_series(DashedLineSeries::new(
                            vec![(x1, y1), (x2, y2)],
                            3,
                            3,
                            style,
                        ))
                        .map_err(|e| e.to_string())?;
                }
            }
        }

        // Draw routers
        for (_, &(rx, ry)) in &router_pos {
            chart
                .draw_series(std::iter::once(Circle::new(
                    (rx, ry),
                    4,
                    &RGBColor(232, 145, 58),
                )))
                .map_err(|e| e.to_string())?;
        }

        // Simple group annotations
        // Group labels (text drawing requires additional font features in plotters)
        // if g > 0 && g <= 16 { ... Text ... }

        root.present().map_err(|e| format!("present: {e}"))?;
    }

    use image::{ImageBuffer, Rgb};
    let img: ImageBuffer<Rgb<u8>, _> =
        ImageBuffer::from_raw(width, height, buffer).ok_or("buffer to image failed")?;
    img.save(output).map_err(|e| e.to_string())?;

    Ok(())
}
