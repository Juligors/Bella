use bevy::{
    prelude::*,
    render::render_resource::{
        Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
    },
};
use bevy_egui::{
    egui::{self, ColorImage, TextureOptions, Widget},
    EguiContexts, EguiPlugin, EguiUserTextures,
};
use plotters::prelude::*;

pub struct PlotsPlugin;
impl Plugin for PlotsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin)
            .add_systems(Startup, setup)
            .add_systems(Update, (simple_plot, modify));
    }
}

#[derive(Resource, Deref, DerefMut)]
struct PlotPreviewImage(Handle<Image>);

fn setup(
    mut cmd: Commands,
    mut egui_user_textures: ResMut<EguiUserTextures>,
    mut images: ResMut<Assets<Image>>,
) {
    let (width, height) = (1000u32, 1000u32);
    let buffer = vec![255u8; width as usize * height as usize * 3];

    let size = Extent3d {
        width,
        height,
        ..default()
    };
    let mut image = Image {
        texture_descriptor: TextureDescriptor {
            label: None,
            size,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8Unorm,
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        },
        data: buffer,
        ..default()
    };
    image.resize(size);
    // image.data[0] = image.data[1];

    let image_handle = images.add(image);
    egui_user_textures.add_image(image_handle.clone());
    cmd.insert_resource(PlotPreviewImage(image_handle.clone()));
}

fn modify(plot_preview: Res<PlotPreviewImage>, mut kurwa: ResMut<Assets<Image>>) {
    let (width, height) = (1000u32, 1000u32);
    let mut buffer = vec![255u8; width as usize * height as usize * 3];

    let root = BitMapBackend::with_buffer(&mut buffer, (1000, 1000)).into_drawing_area();
    root.fill(&RGBColor(255, 0, 0)).unwrap();
    root.draw(&plotters::element::Circle::new(
        (40, 40),
        30,
        Into::<ShapeStyle>::into(RGBColor(0, 255, 0)).filled(),
    ))
    .unwrap();
    root.present().unwrap();
    drop(root);

    let k = kurwa.get_mut(plot_preview.0.clone()).unwrap();
    // Fix that stupid library's output
    (0..(buffer.len() / 3))
        .flat_map(|i| [buffer[3 * i], buffer[3 * i + 1], buffer[3 * i + 2], 255])
        .enumerate()
        .for_each(|(i, x)| k.data[i] = x);
}

fn simple_plot(
    mut ctxs: EguiContexts,
    plot_preview: Res<PlotPreviewImage>,
    mut kurwa: ResMut<Assets<Image>>,
) {
    let plot_preview_id = ctxs
        .image_id(&plot_preview)
        .expect("Can't find plot image.");

    let k = kurwa.get_mut(plot_preview.0.clone()).unwrap();
    // k.data = vec![];

    // let size = Extent3d {
    //     width,
    //     height,
    //     ..default()
    // };
    // let mut image = Image {
    //     texture_descriptor: TextureDescriptor {
    //         label: None,
    //         size,
    //         dimension: TextureDimension::D2,
    //         format: TextureFormat::Rgba8Unorm,
    //         mip_level_count: 1,
    //         sample_count: 1,
    //         usage: TextureUsages::TEXTURE_BINDING
    //             | TextureUsages::COPY_DST
    //             | TextureUsages::RENDER_ATTACHMENT,
    //         view_formats: &[],
    //     },
    //     data: buffer,
    //     ..default()
    // };

    // let (width, height) = (100u32, 100u32);
    // let mut buffer = vec![255u8; width as usize * height as usize * 4];

    // let root = BitMapBackend::with_buffer(&mut buffer, (width, height)).into_drawing_area();
    // root.fill(&RGBColor(255, 0, 0)).unwrap();
    // root.draw(&plotters::element::Circle::new(
    //     (40, 40),
    //     30,
    //     Into::<ShapeStyle>::into(RGBColor(0, 255, 0)).filled(),
    // ))
    // .unwrap();
    // root.present().unwrap();
    // drop(root);

    // Fix that stupid library's output
    // let buffer: Vec<u8> = (0..(global_buffer.len() / 3))
    //     .flat_map(|i| {
    //         [
    //             global_buffer[3 * i],
    //             global_buffer[3 * i + 1],
    //             global_buffer[3 * i + 2],
    //             255,
    //         ]
    //     })
    //     .collect();

    // let handle = ctxs.ctx_mut().load_texture(
    //     "pierdolsie",
    //     ColorImage::from_rgba_unmultiplied([100, 100], &buffer),
    //     TextureOptions::default(),
    // );
    // let texture = egui::load::SizedTexture::from_handle(&handle);

    egui::Window::new("Plot window").show(ctxs.ctx_mut(), |ui| {
        //     ui.image(texture);

        ui.image(egui::load::SizedTexture::new(
            plot_preview_id,
            egui::vec2(300., 300.),
        ))
    });
}
