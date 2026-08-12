use console_error_panic_hook::set_once as set_panic_hook;
use worker::*;

const MAX_SVG_BYTES: usize = 10_000_000;
const MAX_DIMENSION: u32 = 4_096;
const MAX_PIXELS: u64 = 16_777_216;

#[event(fetch)]
pub async fn main(mut req: Request, _env: Env, _ctx: worker::Context) -> Result<Response> {
    set_panic_hook();

    if req.path() != "/render" {
        return Response::error("Not found", 404);
    }
    if req.method() != Method::Post {
        return Response::error("Method not allowed", 405);
    }

    let content_type = req.headers().get("content-type")?.unwrap_or_default();
    if content_type.split(';').next().unwrap_or_default().trim() != "image/svg+xml" {
        return Response::error("Content-Type must be image/svg+xml", 415);
    }

    if let Some(length) = req.headers().get("content-length")? {
        if length.parse::<usize>().unwrap_or(MAX_SVG_BYTES + 1) > MAX_SVG_BYTES {
            return Response::error("SVG exceeds 10 MB", 413);
        }
    }

    let svg_data = req.bytes().await?;
    if svg_data.is_empty() || svg_data.len() > MAX_SVG_BYTES {
        return Response::error("SVG must contain 1 byte to 10 MB", 413);
    }

    let out = match render_svg(&svg_data) {
        Ok(out) => out,
        Err(message) => return Response::error(message, 422),
    };
    let headers = Headers::new();
    headers.set("content-type", "image/png")?;
    headers.set("x-content-type-options", "nosniff")?;
    Ok(Response::from_bytes(out)?.with_headers(headers))
}

fn render_svg(svg_data: &[u8]) -> std::result::Result<Vec<u8>, String> {
    let opt = usvg::Options::default();
    let rtree = usvg::Tree::from_data(&svg_data, &opt.to_ref())
        .map_err(|err| format!("failed to decode SVG: {}", err))?;
    let pixmap_size = rtree.svg_node().size.to_screen_size();
    if pixmap_size.width() > MAX_DIMENSION
        || pixmap_size.height() > MAX_DIMENSION
        || u64::from(pixmap_size.width()) * u64::from(pixmap_size.height()) > MAX_PIXELS
    {
        return Err("SVG dimensions exceed the render limit".into());
    }
    let mut pixmap = tiny_skia::Pixmap::new(pixmap_size.width(), pixmap_size.height())
        .ok_or_else(|| "failed to create PNG buffer".to_string())?;
    resvg::render(
        &rtree,
        usvg::FitTo::Original,
        tiny_skia::Transform::default(),
        pixmap.as_mut(),
    )
    .ok_or_else(|| "failed to render PNG".to_string())?;

    pixmap
        .encode_png()
        .map_err(|err| format!("failed to encode PNG: {}", err))
}

#[cfg(test)]
mod tests {
    use super::render_svg;

    #[test]
    fn renders_svg_to_png() {
        let png = render_svg(br#"<svg xmlns="http://www.w3.org/2000/svg" width="2" height="2"><rect width="2" height="2" fill="red"/></svg>"#).unwrap();
        assert_eq!(&png[..8], &[137, 80, 78, 71, 13, 10, 26, 10]);
    }
}
