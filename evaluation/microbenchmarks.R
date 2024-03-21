source("common.R")

data <- Sys.glob(file.path("..", "shared", "target", "criterion", "*", "*", "*", "new", "estimates.json")) %>%
  purrr::map_df(~(function(file) {
    json <- jsonlite::read_json(file, show_col_types = FALSE)
    parts = stringr::str_match(file, "^.*/([^/]*)/([^/]*)/([^/-]*)-?([0-9]*)/new/estimates.json$")

    tibble(
      group = parts[2],
      protocol = parts[3],
      experiment = parts[4],
      parameter = as.numeric(parts[5]),
      point_estimate = json$mean$point_estimate,
      standard_error = json$mean$standard_error,
      confidence_level = json$mean$confidence_interval$confidence_level,
      lower_bound = json$mean$confidence_interval$lower_bound,
      upper_bound = json$mean$confidence_interval$upper_bound,
    )
  })(.))

  data

y_scale <- 1e-6
y_suffix <- "ms"

file <- "results/microbenchmark"
data <- dplyr::filter(data, protocol == "ParticipateP")
plot <- ggplot2::ggplot(data, ggplot2::aes(x = .data$parameter, y = .data$point_estimate, color = .data$experiment)) +
  ggplot2::geom_line(linetype = "solid", linewidth = .3, alpha = .2) +
  ggplot2::geom_point(shape = 45, size = 4.5) +
  ggplot2::geom_errorbar(
    aes(ymin=.data$lower_bound, ymax=.data$upper_bound),
    width = .3,
    linewidth = .2
  ) +
  theme_custom() +
  ggplot2::labs(
    title = paste(
      "$\\Pi_{\\mathsf{",
      str_replace(data$protocol, "[0-9]$", ""),
      "}}^{",
      str_replace(stringr::str_sub(data$protocol, -1), "[^0-9]*", ""),
      "}$",
      sep = ""
    )
  ) +
  ggplot2::scale_y_continuous(
    labels = scales::label_number(scale = y_scale, suffix = y_suffix),
    limits = c(0, (ceiling(max(data$point_estimate)*(y_scale/10))/(y_scale/10)))
  ) +
  ggplot2::scale_x_continuous(
    limits = c(0, 64),
    breaks = c(0, 8, 16, 24, 32, 40, 48, 56, 64)
  )
plot(plot)
ggsave(paste(file, ".jpg", sep=""), width = 7, height = 3)
dev.off()

exit()

create_plot <- function(data) {
  ggplot2::ggplot(data, ggplot2::aes(x = .data$parameter, y = .data$point_estimate)) +
    ggplot2::geom_line(linetype = "dotted", linewidth = .3, color = color_other) +
    ggplot2::geom_point(shape = 45, size = 4.5) +
    ggplot2::geom_errorbar(
      aes(ymin=.data$lower_bound, ymax=.data$upper_bound),
      width = .3,
      linewidth = .2
    ) +
    theme_custom() +
    ggplot2::labs(
      title = paste(
        "$\\Pi_{\\mathsf{",
        str_replace(data$protocol, "[0-9]$", ""),
        "}}^{",
        str_replace(stringr::str_sub(data$protocol, -1), "[^0-9]*", ""),
        "}$",
        sep = ""
      )
    ) +
    ggplot2::scale_y_continuous(
      labels = scales::label_number(scale = y_scale, suffix = y_suffix),
      limits = c(0, (ceiling(max(data$point_estimate)*(y_scale/10))/(y_scale/10)))
    )
}

plots <- tibble::tribble(
  ~name, ~protocols, ~experiment,
  "register_by_attributes", list("RegisterP1", "RegisterS", "RegisterP2"), "A",
  "participate_by_qualifier", list("ParticipateP", "ParticipateO", "ParticipateS"), "Q",
  "participate_by_qualifier_tags", list("ParticipateP", "ParticipateO", "ParticipateS"), "QT",
  "participate_by_disqualifier", list("ParticipateP", "ParticipateO", "ParticipateS"), "D",
  "participate_by_disqualifier_tags", list("ParticipateP", "ParticipateO", "ParticipateS"), "DT",
  "participate_by_set_constraints", list("ParticipateP", "ParticipateO", "ParticipateS"), "S",
  "participate_by_set_constraint_length", list("ParticipateP", "ParticipateO", "ParticipateS"), "SL",
  "participate_by_range_constraints", list("ParticipateP", "ParticipateO", "ParticipateS"), "R",
  "participate_by_range_constraint_length", list("ParticipateP", "ParticipateO", "ParticipateS"), "RL",
  "payout", list("GetBalance", "PayoutP", "PayoutS"), "L",
  "padding", list("PaddingP1", "PaddingS", "PaddingP2"), "L"
)

for (i in 1:nrow(plots)) {
  file <- paste("results/microbenchmark_", plots$name[[i]], sep = "")
  tikz(file = paste(file, ".tex", sep=""), width = 7, height = 3)
  plotlist <- purrr::map(plots$protocols[[i]], function(x) create_plot(dplyr::filter(data, protocol == x & experiment == plots$experiment[[i]])))
  plot <- plot_grid(plotlist = plotlist, ncol = length(plotlist))
  plot(plot)
  ggsave(paste(file, ".jpg", sep=""), width = 7, height = 3)
  dev.off()
}
