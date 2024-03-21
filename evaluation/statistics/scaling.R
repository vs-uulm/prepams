read_scaling_experiment <- function(glob) {
  Sys.glob(file.path(path, glob, "participations.csv")) %>% purrr::map_df(~(function(file) {
    readr::read_csv(file, show_col_types = FALSE) %>%
      dplyr::mutate(
        run = stringr::str_match(file, "/([^/]*)/([^/]*).csv$")[2],
        experiment = stringr::str_match(file, "/([^/]*)/([^/]*).csv$")[3],
        workload = strtoi(stringr::str_match(file, "([^-0-9]*)-([0-9]+)/([^/]*).csv$")[3]),
        participant = .data$participate,
        participant_size = .data$participate_size,
        organizer = .data$reward,
        organizer_size = .data$reward_size,
      ) %>%
      tidyr::pivot_longer(
        c(participant, organizer),
        names_to = "type",
        values_to = "time"
      )
  })(.)) %>% dplyr::filter(type == "participant" & experiment == "participations")
}

# generate qualifier/disqualifier scaling plot
tikz(
  file = "results/scaling.tex",
  width = plot_width,
  height = 1.25,
  #standAlone = TRUE
)
plot_scaling_experiment <- function(plot, experiment, label, shape, color, limit) {
  data <- read_scaling_experiment(paste(experiment, "*", sep="-"))
  plot + ggplot2::geom_line(
    data = data,
    mapping = ggplot2::aes(x = workload, y = time),
    linetype = "dotted",
    linewidth = .3,
    alpha = .5,
    color = color
  ) + ggplot2::stat_summary(
    data = data,
    mapping = ggplot2::aes(x = workload, y = time),
    color = color,
    fun.min = min,
    fun.max = max,
    fun = median,
    geom = "linerange",
    linewidth = .3
  ) + ggplot2::stat_summary(
    data = data,
    mapping = ggplot2::aes(x = workload, y = time),
    color = color,
    fun = median,
    geom = "point",
    shape = shape,
    size = .4,
  ) + ggplot2::annotate(
    geom = "text",
    x = max(data$workload) + 1,
    y = max(data$time),
    size = 2,
    label = paste("$", label, "$", sep = ""),
    color = color
  ) + ggplot2::annotate(
    geom = "point",
    x = max(data$workload) + .7,
    y = max(data$time),
    size = .8,
    shape = shape,
    color = color
  ) + ggplot2::annotate(
    geom = "rect",
    xmin = max(data$workload) + .5,
    xmax = max(data$workload) + 1.2,
    ymin = max(data$time) - 600,
    ymax = max(data$time) + 600,
    color = color,
    linewidth = 0.2,
    fill = NA
  )

  #labs(tag = paste("=", label, sep="")) +
}

p <- ggplot2::ggplot() +
  theme_custom() +
  labs(
    title = "$\\Pi_{\\mathsf{Participate}}$",
  ) +
  scale_x_continuous(breaks = seq.int(0, 12)) +
  scale_y_continuous(
    labels = scales::label_number(scale = 1e-3, suffix = "s", accuracy = 0.5),
    limits = c(0, 20000),
    expand =  c(0, 0),
  )

p <- plot_scaling_experiment(p, "qualifier", "q", 4, color_red)
p <- plot_scaling_experiment(p, "disqualifier", "d", 3, color_blue)
p <- plot_scaling_experiment(p, "range-constraint", "r", 5, color_orange)
p <- plot_scaling_experiment(p, "set-constraint", "s", 6, color_green)

plot_grid(p,
#  plot_scaling_experiment(ggplot(), "qualifier", "q", 4, color_red, 10000) + labs(
#    title = "$\\Pi_{\\mathsf{Participate}}$",
#  ),
#  plot_scaling_experiment(ggplot(), "disqualifier", "d", 3, color_blue, 5000),
#  plot_scaling_experiment(ggplot(), "range-constraint", "r", 5, color_orange, 15000),
#  plot_scaling_experiment(ggplot(), "set-constraint", "s", 7, color_green, 20000),
  align = "v",
  nrow = 1
)
ggsave("my_plot.jpg", width = plot_width, height = 2.25)
dev.off()
