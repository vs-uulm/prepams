# Load tidyverse libraries
library(cowplot)
library(tidyverse)

nextpow2 <- function(x) {
    if (is.null(x) || length(x) == 0) return(c())
    if (!is.numeric(x) && !is.complex(x))
        stop("Argument 'x' must be a numeric/complex vector/matrix.")

    x[x == 0] <- 1
    return(ceiling(log2(abs(x))))
}

# Read the CSV file with semicolons delimiter
# data <- read_delim("proof_analysis.txt", delim = ";", col_names = c("input_length", "proof_add", "proof_mul"), show_col_types = FALSE) %>% dplyr::mutate(
#     log_length = ceiling(ifelse(.data$input_length > 0, log2(abs(.data$input_length)), 0))
# ) %>% select(input_length, log_length, proof_add, proof_mul)

data <- read_csv("complexity.csv", show_col_types = FALSE) %>% dplyr::mutate(
    log_length = ceiling(ifelse(.data$input_length > 0, log2(abs(.data$input_length)), 0))
)

f_proof_mul <- Vectorize(function(x)
    -1 + 4 * x + 8 * 2^nextpow2(x) + 2 * nextpow2(x)
)

f_proof_add <- Vectorize(function(x)
    7 + 4 * x + 26 * 2^nextpow2(x) + 2 * nextpow2(x)
)

f_verify_mul <- Vectorize(function(x)
    12 + 2 * x + 2 * 2^nextpow2(x) + 2 * nextpow2(x)
)

f_verify_add <- Vectorize(function(x)
    17 + 2 * x + 20 * 2^nextpow2(x) + 2 * nextpow2(x)
)

x_vals <- seq(0, max(data$input_length), 1)
proof_mul <- tibble(x = x_vals, y = f_proof_mul(x_vals)) %>% rename(input_length = x, proof_n_mul = y)
proof_add <- tibble(x = x_vals, y = f_proof_add(x_vals)) %>% rename(input_length = x, proof_n_add = y)
verify_mul <- tibble(x = x_vals, y = f_verify_mul(x_vals)) %>% rename(input_length = x, verify_n_mul = y)
verify_add <- tibble(x = x_vals, y = f_verify_add(x_vals)) %>% rename(input_length = x, verify_n_add = y)

data <- data %>%
  inner_join(proof_add, by = join_by(input_length)) %>%
  inner_join(proof_mul, by = join_by(input_length)) %>%
  inner_join(verify_add, by = join_by(input_length)) %>%
  inner_join(verify_mul, by = join_by(input_length)) %>%
  mutate(
    dadd_proof = proof_add - proof_n_add,
    dmul_proof = proof_mul - proof_n_mul,
    dadd_verify = verify_add - verify_n_add,
    dmul_verify = verify_mul - verify_n_mul
  )

p_add <- ggplot(data, aes(x = input_length, y = proof_add)) +
  geom_point(shape = 1, size=.6, color="blue") +
  geom_line(aes(x = input_length, y = proof_n_add), color="red")

p_mul <- ggplot(data, aes(x = input_length, y = proof_mul)) +
  geom_point(shape = 1, size=.6, color="blue") +
  geom_line(aes(x = input_length, y = proof_n_mul), color="red")

v_add <- ggplot(data, aes(x = input_length, y = verify_add)) +
  geom_point(shape = 1, size=.6, color="blue") +
  geom_line(aes(x = input_length, y = verify_n_add), color="red")

v_mul <- ggplot(data, aes(x = input_length, y = verify_mul)) +
  geom_point(shape = 1, size=.6, color="blue") +
  geom_line(aes(x = input_length, y = verify_n_mul), color="red")

p_dadd <- ggplot(data, aes(x = input_length, y = dadd_proof)) +
  geom_point(shape = 1, size=.6, color="blue")

p_dmul <- ggplot(data, aes(x = input_length, y = dmul_proof)) +
  geom_point(shape = 1, size=.6, color="blue")

v_dadd <- ggplot(data, aes(x = input_length, y = dadd_verify)) +
  geom_point(shape = 1, size=.6, color="blue")

v_dmul <- ggplot(data, aes(x = input_length, y = dmul_verify)) +
  geom_point(shape = 1, size=.6, color="blue")

plot_grid(
  p_mul, v_mul,
  p_add, v_add,
  p_dmul, v_dmul,
  p_dadd, v_dadd,
  align = "v",
  nrow = 4,
  ncol = 2
)
ggsave("complexity.png", width = 20, height = 20, units = "cm")

summary(data$dmul_proof)
summary(data$dadd_proof)
summary(data$dmul_verify)
summary(data$dadd_verify)