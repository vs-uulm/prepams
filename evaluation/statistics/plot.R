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

#register <- read_csv(
#  file.path(path, "performance", "register.csv"),
#  show_col_types = FALSE
#) %>%
#  dplyr::mutate(participant = registerRequest + registerComplete) %>%
#  dplyr::mutate(service = registerVerify) %>%
#  tidyr::pivot_longer(
#    c(participant, service),
#    names_to = "type",
#    values_to = "time"
#  ) %>%
#  dplyr::mutate(experiment = "register")
#
#participations <- read_csv(
#  file.path(path, "performance", "participations.csv"),
#  show_col_types = FALSE
#) %>%
#  dplyr::mutate(participant = participate) %>%
#  dplyr::mutate(organizer = confirm) %>%
#  dplyr::mutate(service = reward) %>%
#  tidyr::pivot_longer(
#    c(participant, organizer, service),
#    names_to = "type",
#    values_to = "time"
#  ) %>%
#  dplyr::mutate(experiment = "participations")
#
#payout <- read_csv(
#  file.path(path, "performance", "payout.csv"),
#  show_col_types = FALSE
#) %>%
#  dplyr::mutate(participant = paddingRequest + payoutRequest) %>%
#  dplyr::mutate(service = paddingResponse + payoutVerify) %>%
#  tidyr::pivot_longer(
#    c(participant, service),
#    names_to = "type",
#    values_to = "time"
#  ) %>%
#  dplyr::mutate(experiment = "payout")

## output basic statistics
## tibble(type = character(), size = numeric(), time = numeric()) %>%
##   dplyr::bind_rows(
##     register %>% transmute(
##       type = "register",
##       size = registerRequest_size,
##       time = registerRequest + registerComplete
##     )
##   ) %>%
##   dplyr::bind_rows(
##     participations %>% transmute(
##       type = "participate",
##       size = participate_size,
##       time = participate
##     )
##   ) %>%
##   dplyr::bind_rows(
##     payout %>% transmute(
##       type = "payout",
##       size = paddingRequest_size + payoutRequest_size,
##       time = paddingRequest + payoutRequest
##     )
##   ) %>%
##   group_by(type) %>%
##   summarise(
##     n = n() / 2,
##     time_mean = mean(time),
##     time_sd = sd(time),
##     size_mean = mean(size),
##     size_sd = sd(size)
##   )
## 
## output basic statistics service
## tibble(type = character(), time = numeric()) %>%
##   dplyr::bind_rows(
##     register %>% transmute(
##       type = "register",
##       time = registerVerify
##     )
##   ) %>%
##   dplyr::bind_rows(
##     participations %>% transmute(
##       type = "participate",
##       time = reward
##     )
##   ) %>%
##   dplyr::bind_rows(
##     payout %>% transmute(
##       type = "payout",
##       time = paddingResponse + payoutVerify
##     )
##   ) %>%
##   group_by(type) %>%
##   summarise(
##     n = n() / 2,
##     time_mean = mean(time),
##     time_sd = sd(time)
##   )

path <- file.path("results-old", "performance")
# path_laptop <- file.path("results-old", "mobile", "Windows-10_Chrome-121000", "performance")
path_phone <- file.path("results-old", "mobile", "Android-10_Chrome-121000", "performance")
# path_macbook <- file.path("results", "mobile", "MacOS-10157_Safari-173", "performance")
path_tablet <- file.path("results-old", "mobile", "MacOS-10157_Safari-172", "performance")
# path_firefox <- file.path("results", "mobile", "Linux-x86_64_Firefox-1210", "performance")
# path_ipad <- file.path("results", "mobile", "MacOS-10156_Safari-154", "performance")
# path_ipadf <- file.path("results", "mobile", "MacOS-1015_Firefox-1220", "performance")

register <- dplyr::bind_rows(
  read_experiment(path_phone, "register", "phone"),
  read_experiment(path_tablet, "register", "tablet"),
  #read_experiment(path_laptop, "register", "laptop"),
  #read_experiment(path_macbook, "register", "macbook"),
  #read_experiment(path_firefox, "register", "firefox"),
  #read_experiment(path_ipad, "register", "ipad"),
  #read_experiment(path_ipadf, "register", "ipadf"),
  read_experiment(path, "register", "")
)

participations <- dplyr::bind_rows(
  read_experiment(path_phone, "participations", "phone"),
  read_experiment(path_tablet, "participations", "tablet"),
  #read_experiment(path_laptop, "participations", "laptop"),
  #read_experiment(path_macbook, "participations", "macbook"),
  #read_experiment(path_firefox, "participations", "firefox"),
  #read_experiment(path_ipad, "participations", "ipad"),
  #read_experiment(path_ipadf, "participations", "ipadf"),
  read_experiment(path, "participations", ""),
)

payout <- dplyr::bind_rows(
  read_experiment(path_phone, "payout", "phone"),
  read_experiment(path_tablet, "payout", "tablet"),
  #read_experiment(path_laptop, "payout", "laptop"),
  #read_experiment(path_macbook, "payout", "macbook"),
  #read_experiment(path_firefox, "payout", "firefox"),
  #read_experiment(path_ipad, "payout", "ipad"),
  #read_experiment(path_ipadf, "payout", "ipadf"),
  read_experiment(path, "payout", ""),
)

print(
    payout %>% dplyr::filter(
      type == "participant phone",
    ) %>% 
   summarise(
     n = n(),
     time_mean = mean(time),
     time_sd = sd(time),
   )
)

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

tikz(
  file = "results/plot.tex",
  width = plot_width,
  height = plot_height,
  packages = c(
    getOption("tikzLatexPackages"),
    "\\usepackage{fontawesome}"
  ),
  #standAlone = TRUE
)
plot(plot_grid(plotlist = list(
  create_plot(register, "$\\Pi_{\\mathsf{Register}}$", 1, "ms"),
  create_plot(participations, "$\\Pi_{\\mathsf{Participate}}$", 1e-3, "s"),
  create_plot(payout, "$\\Pi_{\\mathsf{Payout}}$", 1e-3, "s")
), nrow = 3))
ggsave("my_plot.jpg", width = plot_width, height = plot_height)

dev.off()
unique(participations$type)
