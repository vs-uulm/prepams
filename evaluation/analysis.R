list.of.packages <- c("tikzDevice", "cowplot", "tidyverse", "ggrastr")
new.packages <- list.of.packages[!(list.of.packages %in% installed.packages()[,"Package"])]
if(length(new.packages)) install.packages(new.packages)

library(tikzDevice)
library(tidyverse)
library(cowplot)
library(ggrastr)

base_size = 7
plot_width = 3.3366142
textcolor = "#000000"
color20 = "#f1f3f0"
color50 = "#d1cfc4"
colorRed = "#a32638"
colorBlue = "#26547c"
colorWhite = "#ffffff00"
colorGrey = "#A9A28D"
colorGreyT = "#A9A28D66"

theme_custom <- function () {
    theme_cowplot(font_size = base_size) %+replace%
    theme(
        axis.title.x = element_blank(),
        axis.title.y = element_blank(),
        plot.title = element_text(face="bold", colour = textcolor, size = 8, hjust = 0.5),
        panel.grid.major.y = element_line(color = color50, linewidth = 0.25, linetype = 1),
        axis.ticks = element_line(color = color50, linewidth = 0.25)
    )
}

read_experiment <- function(file) {
    read_csv(file, show_col_types = FALSE) %>%
        mutate(
            run = str_match(file, '/([^/]*)/([^/]*).csv$')[2],
            experiment = str_match(file, '/([^/]*)/([^/]*).csv$')[3],
            participant = ifelse(
                experiment == "register",
                registerRequest + registerComplete, 
                ifelse(experiment == "participations", participate, payoutRequest)
            ),
            service = ifelse(
                experiment == "register",
                registerVerify,
                ifelse(experiment == "participations", reward, payoutVerify)
            ),
        ) %>%
        pivot_longer(
            c(participant, service),
            names_to = "type",
            values_to = "time"
        ) %>%
        mutate(
            type = ifelse(experiment == "participations" & type == "service", "organizer", type),
            size = ifelse(
                type == "participant",
                ifelse(
                    experiment == "register",
                    registerRequest_size + registerComplete_size, 
                    ifelse(experiment == "participations", participate_size, payoutRequest_size)
                ),
                ifelse(
                    experiment == "register",
                    registerVerify_size,
                    ifelse(experiment == "participations", reward_size, payoutVerify_size)
                )
            )
        )
}

data <- Sys.glob(file.path("results", "*qualifier*", "*.csv")) %>% map_df(~read_experiment(.))

# generate qualifier/disqualifier scaling plot
tikz(file = "results/scaling.tex", width = plot_width, height = 1.25, standAlone = TRUE)
plot_grid(
    ggplot()
        + stat_summary(
            data = data %>% filter(type == "participant" & experiment == "participations" & disqualifier == 0),
            mapping = aes(x=qualifier, y=time),
            color = colorRed,
            fun.min = min,
            fun.max = max,
            fun = median,
            geom="linerange",
            linewidth=.3
        )
        + stat_summary(
            data = data %>% filter(type == "participant" & experiment == "participations" & disqualifier == 0),
            mapping = aes(x=qualifier, y=time),
            color = colorRed,
            fun.min = median,
            fun.max = median,
            fun = median,
            geom="errorbar",
            linewidth=.4,
            width=.2
        )
        + stat_summary(
            data = data %>% filter(type == "participant" & experiment == "participations" & qualifier == 0),
            mapping = aes(x=disqualifier, y=time),
            color = colorBlue,
            fun.min = min,
            fun.max = max,
            fun = median,
            geom="linerange",
            linewidth=.3
        )
        + stat_summary(
            data = data %>% filter(type == "participant" & experiment == "participations" &qualifier == 0),
            mapping = aes(x=disqualifier, y=time),
            color = colorBlue,
            fun = median,
            geom="point",
            linewidth=.4,
            shape=4,
            size=.4,
            width=.2
        )
        + theme_custom()
        + labs(title = '$\\Pi_{\\mathsf{Participate}}$')
        + scale_y_continuous(labels = scales::label_number(scale=1e-3, suffix = "s"), limits = c(0, 11000), expand =  c(0, 0))
        + scale_x_continuous(breaks=c(0, 1, 2, 3, 4, 5, 6, 7, 8 , 9, 10, 11, 12)),
    nrow = 1
)
dev.off()

register <- read_csv("results/performance/register.csv", show_col_types = FALSE) %>%
    mutate(participant = registerRequest + registerComplete) %>%
    mutate(service = registerVerify) %>%
    pivot_longer(c(participant, service), names_to = "type", values_to = "time") %>%
    mutate(experiment = "register")

participations <- read_csv("results/performance/participations.csv", show_col_types = FALSE) %>%
    mutate(participant = participate) %>%
    mutate(organizer = reward) %>%
    pivot_longer(c(participant, organizer), names_to = "type", values_to = "time") %>%
    mutate(experiment = "participations")

payout <- read_csv("results/performance/payout.csv", show_col_types = FALSE) %>%
    mutate(participant = payoutRequest) %>%
    mutate(service = payoutVerify) %>%
    pivot_longer(c(participant, service), names_to = "type", values_to = "time") %>%
    mutate(experiment = "payout")

# output basic statistics
tibble(type = character(), size = numeric(), time = numeric()) %>%
    bind_rows(register %>% transmute(type = "register", size = registerRequest_size, time = registerRequest)) %>%
    bind_rows(participations %>% transmute(type = "participate", size = participate_size, time = participate)) %>%
    bind_rows(payout %>% transmute(type = "payout", size = payoutRequest_size, time = payoutRequest)) %>%
    group_by(type) %>%
    summarise(
        n = n(),
        time_mean = mean(time),
        time_sd = sd(time),
        size_mean = mean(size),
        size_sd = sd(size)
    )

# generate main plot
tikz(file = "results/plot.tex", width = plot_width, height = 3, standAlone = TRUE)
plot_grid(
    ggplot(register, aes(x=type, y=time))
        + rasterize(geom_jitter(color=colorGrey, fill=colorGreyT, alpha=.04, size=.4, width = 0.25), dpi=300)
        + geom_violin(linewidth=0.3, fill=colorWhite)
        + stat_summary(fun=median, geom="point", size=1, shape=3, color=colorRed)
        + theme_custom()
        + labs(title = '$\\Pi_{\\mathsf{Register}}$')
        + scale_y_continuous(labels = scales::label_number(suffix = "ms"))
        + scale_x_discrete(labels = c("participant" = "$\\mathsf{P}$", "service" = "$\\mathsf{S}$", "organizer" = "$\\mathsf{O}$"))
        + coord_flip(),
    ggplot(participations, aes(x=type, y=time))
        + rasterize(geom_jitter(color=colorGrey, fill=colorGreyT, alpha=.04, size=.4, width = 0.25), dpi=300)
        + geom_violin(linewidth=0.3, fill=colorWhite)
        + stat_summary(fun=median, geom="point", size=1, shape=3, color=colorRed)
        + theme_custom()
        + labs(title = '$\\Pi_{\\mathsf{Participate}}$')
        + scale_y_continuous(labels = scales::label_number(scale=1e-3, suffix = "s"))
        + scale_x_discrete(labels = c("participant" = "$\\mathsf{P}$", "service" = "$\\mathsf{S}$", "organizer" = "$\\mathsf{O}$"))
        + coord_flip(),
    ggplot(payout, aes(x=type, y=time))
        + rasterize(geom_jitter(color=colorGrey, fill=colorGreyT, alpha=.04, size=.4, width = 0.25), dpi=300)
        + geom_violin(linewidth=0.3, fill=colorWhite)
        + stat_summary(fun=median, geom="point", size=1, shape=3, color=colorRed)
        + theme_custom()
        + labs(title = '$\\Pi_{\\mathsf{Payout}}$')
        + scale_y_continuous(labels = scales::label_number(scale=1e-3, suffix = "s"))
        + scale_x_discrete(labels = c("participant" = "$\\mathsf{P}$", "service" = "$\\mathsf{S}$", "organizer" = "$\\mathsf{O}$"))
        + coord_flip(),
    nrow = 3
)

dev.off()
