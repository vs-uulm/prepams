packages <- c("tikzDevice", "cowplot", "tidyverse", "ggrastr", "jsonlite")
new_packages <- packages[!(packages %in% installed.packages()[, "Package"])]
if (length(new_packages)) install.packages(new_packages)

library(tikzDevice)
library(tidyverse)
library(cowplot)
library(ggrastr)
library(dplyr)
library(ggplot2)

base_size <- 7
plot_width <- 3.3366142
plot_height <- 3
textcolor <- "#000000"
color20 <- "#f1f3f0"
color50 <- "#d1cfc4"
color_other <- "#a9a28d"
color_red <- "#a32638"
color_blue <- "#26547c"
color_white <- "#ffffff00"
color_grey <- "#a9a28d"
color_grey_t <- "#a9a28d66"
color_green <- "#56aa1c"
color_orange <- "#df6c07"

theme_custom <- function() {
  theme_cowplot(font_size = base_size) %+replace%
    ggplot2::theme(
      axis.title.x = ggplot2::element_blank(),
      axis.title.y = ggplot2::element_blank(),
      plot.tag.position = "bottomright",
      plot.tag = ggplot2::element_text(
        colour = textcolor,
        size = 6,
        margin = margin(t = -7)
      ),
      plot.title = ggplot2::element_text(
        face = "bold",
        colour = textcolor,
        size = 8,
        hjust = 0.5
      ),
      panel.grid.major.y = ggplot2::element_line(
        color = color50,
        linewidth = 0.25,
        linetype = 1
      ),
      axis.ticks = ggplot2::element_line(
        color = color50,
        linewidth = 0.25
      )
    )
}
