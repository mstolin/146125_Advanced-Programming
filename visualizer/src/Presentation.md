# Visualizer 

> *This is a visualizer for the Rust project of group SGX*
___

This implementation is using two main libraries in its dependencies

1. [Druid](https://lib.rs/crates/druid) *ver.0.7.0* 
2. [Plotters](https://lib.rs/crates/plotters) *ver 0.3.4*
3. [Serde](https://lib.rs/crates/serde) *ver 1.0.152*

## Druid
>This is one of the many **Rust's GUI Libraries**, and it allows to visualize the program's content as a **Widget application**.

## Plotters
>Plotters is one of many drawing and **data - plotting** libraries; it contains many tools for chart plotting, and it is versatile for both native applications like this one and **WASM** versions too.

## Plotters-Druid
>This library offers a quick method to combine **Plotters** drawings and **Druid**'s GUI, by providing a Widget object to Druid's window which contains the Plot made by Plotters.

## Serde
> Serde is used in this project to allow communication between strategies and the visualizer.  
>  * The **strategies** can **save** their log as a **.json** file;
>  * The **visualizer** can **read** the stored json logs to be rendered;
___

# Widget app
> This part was built using the Druid GUI library; its only purpose is to show the chart plotted on a window;
> **Druid**, with the aid of **Plotters-Druid** made this possible.
# Chart and plot
> Using the library Plotters it was possible to render the input values we want to visualize as dots on the chart connected by a line.  
> This visualizer takes json files containing items of the form \
> `  {
"day": 1.0,
"eur": 171150.75,
"usd": 20091.201,
"yen": 0.0,
"yuan": 25114.344
}` as input . \
>The first value's purpose is to give a chronological order to the operation and represents the x coordinate in the chart, while the currency values are used to plot the y coordinates of the chart.

### Credits
This visualizer was made by Farid Ouedraogo.
