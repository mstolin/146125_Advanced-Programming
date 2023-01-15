use druid::*;
use plotters::prelude::*;
use plotters::style::Color;

use plotters_druid::Plot;

pub fn render_plot(){
    let main_window = WindowDesc::new(chart_builder)
        .title("This is a plot!")
        .window_size((800.0, 600.0));

    AppLauncher::with_window(main_window)
        .launch(())
        .expect("launch failed");
}


fn chart_builder() -> impl Widget<()>{

    Plot::new(|_size, _data, root| {
        let x_values = [0.0f64, 1.15, 2.22, 3.9, 4.11, 5.6, 6.22, 7.04, 8.3, 9.2, 10.54];
        root.fill(&WHITE).unwrap();
        let mut chart_builder = ChartBuilder::on(&root);

        chart_builder.margin(10).set_left_and_bottom_label_area_size(20);

        let mut chart_context = chart_builder.build_cartesian_2d(0.0..15.0, 0.0..150.0).unwrap();
        chart_context.configure_mesh().draw().unwrap();

        chart_context.draw_series(LineSeries::new(x_values.map(|x| (x, x*x)), BLUE.filled())
            .point_size(4)).unwrap();
    })
}