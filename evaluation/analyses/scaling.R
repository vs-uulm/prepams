# helper function to read scaling experiments data
read_scaling_experiment <- function(glob) {
  files <- Sys.glob(file.path("results", glob, "participations.csv"))
  if (length(files) == 0) {
    tibble(
      participate = double(),
      participate_size = double(),
      confirm = double(),
      confirm_size = double(),
      reward = double(),
      reward_size = double(),
      run = character(),
      experiment = character(),
      workload = integer(),
      participant_size  = double(),
      organizer_size = double(),
      type = character(),
      time = double()
    )
  } else {
    files %>% purrr::map_dfr(~(function(file) {
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
}

# generate qualifier/disqualifier scaling plot
plot_scaling_experiment <- function(plot, experiment, label, shape, color, limit) {
  data <- read_scaling_experiment(paste(experiment, "*", sep="-"))
  yOffset <- switch(experiment, qualifier={0}, disqualifier={-1000}, "set-constraint"={2000}, "range-constraint"={1360})
  yOffsetLabel <- switch(experiment, qualifier={300}, disqualifier={-150}, "set-constraint"={-50}, "range-constraint"={-50})
  yOffsetPoint <- switch(experiment, qualifier={-100}, disqualifier={-150}, "set-constraint"={-50}, "range-constraint"={-150})
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
  ) + ggplot2::annotate(
    geom = "rect",
    xmin = max(data$workload) + .5,
    xmax = max(data$workload) + 1.2,
    ymin = max(data$time) - 1600 + yOffset,
    ymax = max(data$time) + 1300 + yOffset,
    color = color,
    linewidth = 0.2,
    fill = "white"
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
    y = max(data$time) + yOffset + yOffsetLabel,
    size = 2,
    label = paste("$", label, "$", sep = ""),
    color = color
  ) + ggplot2::annotate(
    geom = "point",
    x = max(data$workload) + .7,
    y = max(data$time) + yOffset + yOffsetPoint,
    size = .8,
    shape = shape,
    color = color
  )
}

# setup main tex output
tikz(
  file = "results/scaling.tex",
  width = plot_width,
  height = 1.25,
  standAlone = TRUE
)

p <- ggplot2::ggplot() +
  theme_custom() +
  labs(title = "$\\Pi_{\\mathsf{Participate}}$") +
  scale_x_continuous(breaks = seq.int(0, 9)) +
  scale_y_continuous(
    labels = scales::label_number(scale = 1e-3, suffix = "s", accuracy = 0.5),
    limits = c(0, 35000),
    expand =  c(0, 0),
  )
p <- plot_scaling_experiment(p, "qualifier", "q", 4, color_red)
p <- plot_scaling_experiment(p, "disqualifier", "d", 3, color_blue)
p <- plot_scaling_experiment(p, "range-constraint", "r", 5, color_orange)
p <- plot_scaling_experiment(p, "set-constraint", "s", 6, color_green)

plot(p)
dev.off()
