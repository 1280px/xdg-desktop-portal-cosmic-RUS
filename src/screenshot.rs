#![allow(dead_code, unused_variables)]

use smithay::backend::renderer::multigpu::{egl::EglGlesBackend, GpuManager};
use std::{collections::HashMap, fs, io};
use zbus::zvariant;

use crate::wayland::WaylandHelper;
use crate::PortalResponse;

// TODO save to /run/user/$UID/doc/ with document portal fuse filesystem?

#[derive(zvariant::DeserializeDict, zvariant::Type)]
#[zvariant(signature = "a{sv}")]
struct ScreenshotOptions {
    modal: Option<bool>,
    interactive: Option<bool>,
}

#[derive(zvariant::SerializeDict, zvariant::Type)]
#[zvariant(signature = "a{sv}")]
struct ScreenshotResult {
    uri: String,
}

#[derive(zvariant::SerializeDict, zvariant::Type)]
#[zvariant(signature = "a{sv}")]
struct PickColorResult {
    color: (f64, f64, f64), // (ddd)
}

pub struct Screenshot {
    wayland_helper: WaylandHelper,
}

impl Screenshot {
    pub fn new(wayland_helper: WaylandHelper) -> Self {
        Self { wayland_helper }
    }
}

#[zbus::dbus_interface(name = "org.freedesktop.impl.portal.Screenshot")]
impl Screenshot {
    async fn screenshot(
        &self,
        #[zbus(connection)] connection: &zbus::Connection,
        handle: zvariant::ObjectPath<'_>,
        app_id: &str,
        parent_window: &str,
        option: ScreenshotOptions,
    ) -> PortalResponse<ScreenshotResult> {
        // connection.object_server().at(&handle, Request);

        // TODO create handle, show dialog
        // XXX
        //

        let (mut exporter, output) = if let Some(exporter) = self.wayland_helper.dmabuf_exporter() {
            // XXX way to select best output? Multiple?
            if let Some(output) = self.wayland_helper.outputs().first().cloned() {
                (exporter, output)
            } else {
                eprintln!("No output");
                return PortalResponse::Other;
            }
        } else {
            eprintln!("No dmabuf exporter");
            return PortalResponse::Other;
        };

        let res = tokio::task::spawn_blocking(move || -> anyhow::Result<()> {
            let frame = exporter.capture_output(&output, false)?;
            let file = io::BufWriter::new(fs::File::create("/tmp/out.png")?);
            let mut gpu_manager = GpuManager::new(EglGlesBackend, None)?;
            frame.write_to_png(&mut gpu_manager, file)?;
            Ok(())
        })
        .await;

        if let Err(err) = res {
            eprintln!("Failed to capture screenshot: {}", err);
            return PortalResponse::Other;
        }

        // connection.object_server().remove::<Request, _>(&handle);
        PortalResponse::Success(ScreenshotResult {
            uri: format!("file:///tmp/out.png"),
        })
    }

    async fn pick_color(
        &self,
        handle: zvariant::ObjectPath<'_>,
        app_id: &str,
        parent_window: &str,
        option: HashMap<String, zvariant::Value<'_>>,
    ) -> PortalResponse<PickColorResult> {
        // TODO create handle
        // XXX
        PortalResponse::Success(PickColorResult {
            color: (1., 1., 1.),
        })
    }
}
