//! Add a stamp: the palette and placing stamps on the page.

use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use pdfcraft_ui_egui::{PdfCraftApp, QuickTool};

#[test]
fn choosing_and_placing_stamps() {
    let mut h = Harness::builder().with_size(egui::vec2(1400.0, 900.0)).build_eframe(|_cc| {
        let mut app = PdfCraftApp::new();
        app.open_bytes("form.pdf", None, include_bytes!("data/form.pdf").to_vec()).unwrap();
        app.set_option("zoom", "150").unwrap();
        app
    });
    h.run_steps(6);
    assert!(h.state_mut().execute("comment.stamp"));
    h.run_steps(2);
    h.get_by_label("Add a stamp");
    h.get_all_by_label("APPROVED").next().expect("dynamic approved").click();
    h.run_steps(2);
    assert_eq!(h.state().quick_tool, QuickTool::Stamp(pdfcraft_engine::StampKind::DynApproved));
    let p = {
        let r = h.state().views[0].page_screen_rect(0).unwrap();
        r.min + egui::vec2(r.width() * 0.5, r.height() * 0.85)
    };
    h.hover_at(p);
    h.run_steps(1);
    h.drag_at(p);
    h.run_steps(1);
    h.drop_at(p);
    h.run_steps(4);
    let s = h.state();
    let doc = s.session.get(s.views[0].id).unwrap();
    let stamps: Vec<_> = doc.info.annotations.iter().filter(|a| a.subtype == "Stamp").collect();
    assert_eq!(stamps.len(), 1);
    assert_eq!(s.quick_tool, QuickTool::Select, "back to selecting after one stamp");
    for _ in 0..30 {
        h.run_steps(2);
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    if let Ok(dir) = std::env::var("PDFCRAFT_SHOTS") {
        h.render().unwrap().save(format!("{dir}/stamps.png")).unwrap();
    }
}

#[test]
fn creating_placing_and_keeping_custom_stamps() {
    let mut h = Harness::builder().with_size(egui::vec2(1400.0, 900.0)).build_eframe(|_cc| {
        let mut app = PdfCraftApp::new();
        app.open_bytes("form.pdf", None, include_bytes!("data/form.pdf").to_vec()).unwrap();
        app.set_option("zoom", "150").unwrap();
        app
    });
    h.run_steps(6);
    assert!(h.state_mut().execute("comment.stamp"));
    h.run_steps(2);
    // An image becomes a stamp: Create ▸ category and name.
    let mut png = Vec::new();
    image::RgbImage::from_pixel(60, 20, image::Rgb([20, 120, 220])).write_to(&mut std::io::Cursor::new(&mut png), image::ImageFormat::Png).unwrap();
    h.state_mut().start_custom_stamp("logo.png".into(), png);
    h.run_steps(2);
    h.get_by_label("Create Custom Stamp");
    h.state_mut().stamp_draft.category = "Company".into();
    h.run_steps(1);
    h.get_by_label("OK").click();
    h.run_steps(2);
    assert_eq!(h.state().quick_tool, QuickTool::CustomStamp(0));
    h.get_by_label("Company");
    h.get_by_label("logo");
    // Place it.
    let p = {
        let r = h.state().views[0].page_screen_rect(0).unwrap();
        r.min + egui::vec2(r.width() * 0.5, r.height() * 0.85)
    };
    h.hover_at(p);
    h.run_steps(1);
    h.drag_at(p);
    h.run_steps(1);
    h.drop_at(p);
    h.run_steps(4);
    let s = h.state();
    let doc = s.session.get(s.views[0].id).unwrap();
    let stamps: Vec<_> = doc.info.annotations.iter().filter(|a| a.subtype == "Stamp").collect();
    assert_eq!(stamps.len(), 1);
    let r = stamps[0].rect;
    assert!(((r[2] - r[0]) / (r[3] - r[1]) - 3.0).abs() < 0.05, "keeps the picture's shape: {r:?}");
    assert_eq!(doc.can_undo(), Some("Add stamp"));
    // The library is kept with the app's settings.
    let saved = s.persist();
    let mut again = PdfCraftApp::new();
    again.restore(&saved);
    assert_eq!(again.custom_stamps.len(), 1);
    assert_eq!((again.custom_stamps[0].category.as_str(), again.custom_stamps[0].name.as_str()), ("Company", "logo"));
    assert_eq!(again.custom_stamps[0].data, s.custom_stamps[0].data);
}

/// A custom stamp (a company chop) is selected once placed, resizes from its corners only and
/// keeps its picture's proportions; Shift stretches it freely.
#[test]
fn custom_stamps_resize_from_corners_keeping_their_shape() {
    let mut h = Harness::builder().with_size(egui::vec2(1400.0, 900.0)).build_eframe(|_cc| {
        let mut app = PdfCraftApp::new();
        // A blank page: nothing under the stamp but the stamp.
        let blank = b"%PDF-1.7\n1 0 obj << /Type /Catalog /Pages 2 0 R >> endobj\n2 0 obj << /Type /Pages /Kids [3 0 R] /Count 1 /MediaBox [0 0 300 400] >> endobj\n3 0 obj << /Type /Page /Parent 2 0 R >> endobj\ntrailer << /Root 1 0 R >>\n%%EOF";
        app.open_bytes("blank.pdf", None, blank.to_vec()).unwrap();
        app.set_option("zoom", "150").unwrap();
        app
    });
    h.run_steps(6);
    let mut png = Vec::new();
    image::RgbImage::from_pixel(60, 20, image::Rgb([20, 40, 160])).write_to(&mut std::io::Cursor::new(&mut png), image::ImageFormat::Png).unwrap();
    h.state_mut().start_custom_stamp("chop.png".into(), png);
    h.run_steps(2);
    h.get_by_label("OK").click();
    h.run_steps(2);
    let center = h.state().views[0].page_screen_rect(0).unwrap().center();
    h.hover_at(center);
    h.run_steps(1);
    h.drag_at(center);
    h.run_steps(1);
    h.drop_at(center);
    h.run_steps(4);
    let stamp = |h: &Harness<'static, PdfCraftApp>| {
        let s = h.state();
        let doc = s.session.get(s.views[0].id).unwrap();
        doc.info.annotations.iter().find(|a| a.subtype == "Stamp").cloned().expect("the stamp")
    };
    let a = stamp(&h);
    assert_eq!(a.stamp_picture_size, Some([60.0, 20.0]));
    assert_eq!(h.state().views[0].comments.selected, Some((0, a.index)), "selected once placed, handles showing");
    // User space → screen, as the canvas maps it.
    let at = |h: &Harness<'static, PdfCraftApp>, x: f32, y: f32| {
        let s = h.state();
        let page = &s.session.get(s.views[0].id).unwrap().info.pages[0];
        let xf = pdfcraft_ui_egui::canvas::PageXform {
            rect: s.views[0].page_screen_rect(0).expect("on screen"),
            rot: s.views[0].rotation,
            pw: page.width,
            ph: page.height,
        };
        let p = page.user_to_view(x, y);
        xf.norm_to_screen(p[0] / xf.pw, p[1] / xf.ph)
    };
    // Drag between two points in user space.
    let drag = |h: &mut Harness<'static, PdfCraftApp>, from: [f32; 2], to: [f32; 2]| {
        let (from, to) = (at(h, from[0], from[1]), at(h, to[0], to[1]));
        h.hover_at(from);
        h.run_steps(1);
        h.drag_at(from);
        h.run_steps(1);
        h.hover_at(to);
        h.run_steps(2);
        h.drop_at(to);
        h.run_steps(3);
    };
    let ratio = |r: [f32; 4]| (r[2] - r[0]) / (r[3] - r[1]);
    // The bottom-right corner, pulled mostly sideways: it grows but stays 3:1.
    let r = a.rect;
    drag(&mut h, [r[2], r[1]], [r[2] + 30.0, r[1] - 2.0]);
    let r2 = stamp(&h).rect;
    assert!(r2[2] - r2[0] > r[2] - r[0] + 20.0, "bigger: {r:?} → {r2:?}");
    assert!((ratio(r2) - 3.0).abs() < 0.05, "keeps 3:1: {r2:?}");
    assert_eq!(h.state().session.get(h.state().views[0].id).unwrap().can_undo(), Some("Resize comment"));
    // No side handles: the right edge's middle moves the stamp instead of stretching it.
    let mid = (r2[1] + r2[3]) / 2.0;
    drag(&mut h, [r2[2] - 1.0, mid], [r2[2] + 19.0, mid]);
    let r3 = stamp(&h).rect;
    assert!(((r3[2] - r3[0]) - (r2[2] - r2[0])).abs() < 0.5 && ((r3[3] - r3[1]) - (r2[3] - r2[1])).abs() < 0.5, "same size: {r2:?} → {r3:?}");
    assert!((r3[0] - r2[0] - 20.0).abs() < 0.5, "moved: {r2:?} → {r3:?}");
    // Shift stretches freely.
    h.event(egui::Event::ModifiersChanged(egui::Modifiers::SHIFT));
    drag(&mut h, [r3[2], r3[1]], [r3[2] + 30.0, r3[1]]);
    h.event(egui::Event::ModifiersChanged(egui::Modifiers::NONE));
    h.run_steps(2);
    let r4 = stamp(&h).rect;
    assert!(ratio(r4) > 3.5, "stretched: {r4:?}");
    assert!(((r4[3] - r4[1]) - (r3[3] - r3[1])).abs() < 0.5, "only wider: {r3:?} → {r4:?}");
}
