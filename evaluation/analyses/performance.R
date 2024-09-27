# helper function to load performance experiments
read_experiment <- function(path, type, filter) {
  data <- read_csv(
    file.path(path, paste(type, ".csv", sep = "")),
    show_col_types = FALSE
  )

  if (type == "register") {
    data <- dplyr::mutate(
      data,
      participant = registerRequest + registerComplete,
      service = registerVerify
    ) %>% tidyr::pivot_longer(c(participant, service))
  } else if (type == "participations") {
    data <- dplyr::mutate(
      data,
      participant = participate,
      organizer = confirm,
      service = reward
    ) %>% tidyr::pivot_longer(c(participant, organizer, service))
  } else if (type == "payout") {
    data <- dplyr::mutate(
      data,
      participant = paddingRequest + payoutRequest,
      service = paddingResponse + payoutVerify
    ) %>% tidyr::pivot_longer(c(participant, service))
  }

  if (filter != "") {
    data <- data %>%
      dplyr::filter(name == "participant") %>%
      dplyr::mutate(name = paste("participant", filter))
  }

  return (
    data %>%
      dplyr::rename(type = name, time = value) %>%
      dplyr::mutate(experiment = type)
  )
}

# load all experiments
for (path in list.dirs(path = "results", full.names = TRUE, recursive = TRUE)) {
  if (!endsWith(path, "/performance")) {
    next
  }

  for (experiment in c("register", "participations", "payout")) {
    deviceType <- replace_na(str_split_i(str_split_i(path, "/", 3), "_", -1), "")
    data <- read_experiment(path, experiment, deviceType)

    if (exists(experiment)) {
      assign(experiment, dplyr::bind_rows(get(experiment), data))
    } else {
      assign(experiment, data)
    }
  }
}

## generate main plot
create_plot <- function(data, title, y_scale, y_suffix) {
  ggplot2::ggplot(data, ggplot2::aes(x = .data$type, y = .data$time)) +
    rasterize(
      ggplot2::geom_jitter(
        color = color_grey,
        fill = color_grey_t,
        alpha = .04,
        size = .4,
        width = 0.25
      ), dpi = 300
    ) +
    ggplot2::geom_violin(linewidth = 0.3, fill = color_white) +
    ggplot2::stat_summary(
      fun = median,
      geom = "point",
      size = 1,
      shape = 3,
      color = color_red
    ) +
    theme_custom() +
    ggplot2::labs(
      title = paste(title, " {\\small$(n=", nrow(dplyr::filter(data, type == "participant")), ")$}", sep = "")
    ) +
    ggplot2::scale_y_continuous(
      labels = scales::label_number(scale = y_scale, suffix = y_suffix),
      limits = c(0, NA)
    ) +
    ggplot2::coord_flip() +
    ggplot2::scale_x_discrete(
      limits = rev(unique(data$type)),
      labels = c(
        "participant" = "$\\mathsf{P}_\\textrm{\\makebox[1.5em][c]{\\faLaptop}}$",
        "participant phone" = "$\\mathsf{P}_\\textrm{\\makebox[1.5em][c]{\\faMobile}}$",
        "participant tablet" = "$\\mathsf{P}_\\textrm{\\makebox[1.5em][c]{\\faTablet}}$",
        "service" = "$\\mathsf{S}$",
        "organizer" = "$\\mathsf{O}$"
      )
    )
}

# setup main tex output
tikz(
  file = "results/performance.tex",
  width = plot_width,
  height = plot_height,
  packages = c(
    getOption("tikzLatexPackages"),
    "\\usepackage{fontawesome}"
  ),
  standAlone = TRUE
)

plot(plot_grid(plotlist = list(
  create_plot(register, "$\\Pi_{\\mathsf{Register}}$", 1, "ms"),
  create_plot(participations, "$\\Pi_{\\mathsf{Participate}}$", 1e-3, "s"),
  create_plot(payout, "$\\Pi_{\\mathsf{Payout}}$", 1e-3, "s")
), nrow = 3))

dev.off()
unique(participations$type)
