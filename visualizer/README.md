# Purpose :
> This is a visualizer for the data generated by the program. It is a simple widget that can render the data generated by the strategies.

# Curiosities for the use:
> This visualizer works by reading json files created with SERDE;
> Interacting with the visualizer is possible by using the function 
> `save(activities : &[(i32,f64)],filename : &str);`\
> which will create a .json file into a folder where the visualizer can read.\
> 
> If the signature of the function seem confusing, you can simply see it as a vector of tuples of (i32,f64) : the first element is the n° of the operation, while the second one shows the balance of the trader after the operation was executed.

# How to use :
> The function save takes as input a Vec<(i32,f64)> and the desired name of file,
> ** *(no extension needed in it)* **. \

# Example :

# Testing :
> The visualizer is tested by using the function `save` and then checking if the file was created.\ 
> For the plotting the only test is to check if the plot renders correctly.

# Result :

