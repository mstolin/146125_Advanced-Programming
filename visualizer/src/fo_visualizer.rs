use std::error::Error;
use druid::{AppLauncher,Widget,WindowDesc};
use plotters::prelude::*;
use plotters::prelude::full_palette::CYAN_900;
use plotters_druid::Plot;

pub fn render_plot()-> Result<(),Box<dyn Error>>{
    let main_window = WindowDesc::new(chart_builder)
        .title("This is a plot!")
        .window_size((800.0, 600.0));

    AppLauncher::with_window(main_window)
        .launch(())
        .expect("launch failed");
    Ok(())
}


fn chart_builder() -> impl Widget<()>{
    let trades = vec![(0.0, 10.0), (5.0, 20.0), (8.0, 7.0),(8.7, 60.3)];
    Plot::new(move |_size, _data, root| {
        //the values I want to render, (as example)

        //Background color
        root.fill(&WHITE).unwrap();
        //The chart will be put on the window
        let mut chart = ChartBuilder::on(&root);

        //Shifting the chart away from the window borders
        chart
            .margin(10)
            .set_left_and_bottom_label_area_size(50);

        //I need this to draw inside the area of the chart
        let mut chart_context = chart.build_cartesian_2d(0.0..9.0, 0.0..100.0).unwrap();

        //Background grid
        chart_context.configure_mesh().draw().unwrap();

        //to draw a line connecting all the values
        chart_context.draw_series(LineSeries::new(
            trades.clone(),
            &CYAN_900,
        )).unwrap();

        //to have a labelled point on every value update on the plot
        chart_context.draw_series(PointSeries::of_element(
            trades.clone(),
            5,
            &CYAN_900,
            &|c, s, st| {
                 EmptyElement::at(c)    // Composed element on-the-fly
                    + Circle::new((0,0),s,st.filled()) // New pixel coordinate is established in c, with a filled circle on the point
                    + Text::new(format!("{:?}", c.1), (0, -20), ("sans-serif", 16).into_font()) //Every point will have the label showing the 2nd value of the tuple
            },
        )).unwrap();

    })
}