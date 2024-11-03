use super::{
    organism::{plant::PlantMarker, ReproductionState},
    time::HourPassedEvent,
};
use bevy::{
    prelude::*,
    render::render_resource::{
        Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
    },
};
use bevy_egui::{egui, EguiContexts, EguiPlugin, EguiUserTextures};
use plotters::prelude::*;

pub struct PlotsPlugin;
impl Plugin for PlotsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin)
            .add_systems(Startup, setup)
            .add_systems(
                Update,
                (
                    simple_plot,
                    plot_data,
                    update_plant_plot_data.run_if(on_event::<HourPassedEvent>()),
                ),
            );
    }
}

#[derive(Resource, Deref, DerefMut)]
struct PlotPreviewImage(Handle<Image>);

#[derive(Resource)]
pub struct PlantPlot {
    width: u32,
    height: u32,
    buffer: Vec<u8>,
    pub y_data_developing: Vec<usize>,
    pub y_data_ready_to_reproduce: Vec<usize>,
    pub y_data_waiting_to_reproduce: Vec<usize>,
}

fn setup(
    mut cmd: Commands,
    mut egui_user_textures: ResMut<EguiUserTextures>,
    mut images: ResMut<Assets<Image>>,
) {
    let (width, height) = (1000u32, 1000u32);
    let my_plot = PlantPlot {
        width,
        height,
        buffer: vec![255u8; width as usize * height as usize * 3],
        y_data_developing: vec![],
        y_data_ready_to_reproduce: vec![],
        y_data_waiting_to_reproduce: vec![],
    };

    let size = Extent3d {
        width,
        height,
        ..default()
    };
    let image = Image {
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
        data: vec![255u8; width as usize * height as usize * 4],
        ..default()
    };
    cmd.insert_resource(my_plot);

    let image_handle = images.add(image);
    egui_user_textures.add_image(image_handle.clone());
    cmd.insert_resource(PlotPreviewImage(image_handle.clone()));
}

fn plot_data(
    plot_preview: Res<PlotPreviewImage>,
    mut images: ResMut<Assets<Image>>,
    mut plant_plot: ResMut<PlantPlot>,
) {
    let PlantPlot {
        width,
        height,
        buffer,
        y_data_developing,
        y_data_ready_to_reproduce,
        y_data_waiting_to_reproduce,
    } = plant_plot.as_mut();

    let max_0 = y_data_developing
        .iter()
        .max_by(|x, y| x.cmp(y))
        .unwrap_or(&1);
    let max_1 = y_data_ready_to_reproduce
        .iter()
        .max_by(|x, y| x.cmp(y))
        .unwrap_or(&1);
    let max_2 = y_data_waiting_to_reproduce
        .iter()
        .max_by(|x, y| x.cmp(y))
        .unwrap_or(&1);
    let max_all = **[max_0, max_1, max_2]
        .iter()
        .max()
        .expect("Can't get max all value");

    let x_spec = 0f32..(y_data_developing.len() as f32 * 1.1);
    let y_spec = 0f32..(max_all as f32 * 1.1);

    let root = BitMapBackend::with_buffer(buffer, (*width, *height)).into_drawing_area();
    root.fill(&WHITE).unwrap();

    let mut chart = ChartBuilder::on(&root)
        .caption("Number of plants", ("Arial", 40))
        // .margin(20)
        // .set_all_label_area_size(50)
        .set_label_area_size(LabelAreaPosition::Bottom, 50)
        .set_label_area_size(LabelAreaPosition::Left, 80)
        .build_cartesian_2d(x_spec, y_spec)
        .expect("Can't build plot.");

    chart
        .configure_mesh()
        .light_line_style(TRANSPARENT)
        .label_style(("Arial", 30, &BLACK))
        .draw()
        .expect("Can't draw plot.");

    chart
        .draw_series(LineSeries::new(
            y_data_developing
                .iter()
                .enumerate()
                .map(|(i, data)| (i as f32, *data as f32)),
            &RED,
        ))
        .expect("Can't draw red")
        .label("Developing");

    chart
        .draw_series(LineSeries::new(
            y_data_ready_to_reproduce
                .iter()
                .enumerate()
                .map(|(i, data)| (i as f32, *data as f32)),
            &GREEN,
        ))
        .expect("Can't draw green")
        .label("Ready to reproduce");

    chart
        .draw_series(LineSeries::new(
            y_data_waiting_to_reproduce
                .iter()
                .enumerate()
                .map(|(i, data)| (i as f32, *data as f32)),
            &BLUE,
        ))
        .expect("Can't draw blue")
        .label("Waiting to reproduce");

    chart
        .configure_series_labels()
        .background_style(WHITE)
        .border_style(BLACK)
        .position(SeriesLabelPosition::MiddleRight)
        .label_font(("Arial", 30))
        .draw()
        .expect("Can't draw plot.");

    root.present().unwrap();
    // TODO: if we move it to util function we shouldn't have to use drops
    drop(chart);
    drop(root);

    let image = images.get_mut(plot_preview.0.clone().id()).unwrap();

    // Fix that stupid library's output and put data into image
    (0..(buffer.len() / 3))
        .flat_map(|i| [buffer[3 * i], buffer[3 * i + 1], buffer[3 * i + 2], 255])
        .enumerate()
        .for_each(|(i, x)| image.data[i] = x);
}

fn simple_plot(mut ctxs: EguiContexts, plot_preview: Res<PlotPreviewImage>) {
    let plot_preview_id = ctxs
        .image_id(&plot_preview)
        .expect("Can't find plot image.");

    egui::Window::new("Plot window")
        .default_open(false)
        .show(ctxs.ctx_mut(), |ui| {
            ui.image(egui::load::SizedTexture::new(
                plot_preview_id,
                egui::vec2(600., 500.),
            ))
        });
}

fn update_plant_plot_data(
    mut plot: ResMut<PlantPlot>,
    plants: Query<&ReproductionState, With<PlantMarker>>,
) {
    let mut developing = 0;
    let mut ready_to_reproduce = 0;
    let mut waiting_to_reproduce = 0;

    plants
        .iter()
        .for_each(|reproduction_state| match reproduction_state {
            ReproductionState::Developing(_) => developing += 1,
            ReproductionState::ReadyToReproduce => ready_to_reproduce += 1,
            ReproductionState::WaitingToReproduce(_) => waiting_to_reproduce += 1,
        });

    plot.y_data_developing.push(developing);
    plot.y_data_ready_to_reproduce.push(ready_to_reproduce);
    plot.y_data_waiting_to_reproduce.push(waiting_to_reproduce);
}
