# PrePaMS: A Privacy-preserving Participant Management System

This repository contains source code artifacts, experiments, and results associated with our PETS'22 submission.

## Repository Structure
 * [`shared/`](shared) - a rust-based webassembly module implementing the PrePaMS schema.
 * [`backend/`](backend) - the PrePaMS server application that interfaces with the PrePaMS web application.
 * [`frontend/`](frontend) - the main client-side source code of the PrePaMS web application.
 * [`evaluation/`](evaluation) - evaluation artifacts to reproduce our performance evluation results (see [Evaluation](#evaluation)).
 * [`results/`](results) - our evaluation results as reported in our submission.

## Abstract
> Taking part in surveys, experiments, and studies is often compensated by rewards to increase the number of participants and encourage attendance.
> While privacy requirements are usually considered for the actual participation, privacy aspects of the reward procedure are mostly ignored so far.
> To this end, we introduce PrePaMS, an efficient participation management system that supports prerequisite checks and reward procedures in a privacy-preserving way.
> Our system organizes participations with potential (dis-)qualifying dependencies and enables secure reward payoffs.
> By leveraging a set of proven cryptographic primitives and mechanisms such as anonymous credentials and zero-knowledge proofs, participations are protected so that service providers and organizers cannot derive the identity of participants even within the reward process.
> In this paper, we have designed and implemented a prototype of PrePaMS to show its effectiveness and we have evaluated its performance under realistic workloads.
> PrePaMS covers the information whether subjects have participated in surveys, experiments, or studies.
> When combined with other secure solutions for the actual data collection within these events, PrePaMS can represent a cornerstone for more privacy-preserving empirical research.
>
> ![](results/overview.png)

## Evaluation
To assess the practicability of our proof-of-concept prototype, we have evaluated the performance of the PrePaMS primitives.
The evaluation in this repository follows the [Popper convention](https://getpopper.io/) for reproducible evaluations.

> ![](results/plot.png)
>
> **Figure 1:** Violin/jitter plots of measured execution times in seconds of our PrePaMS proof of concept implementation based on a synthetic workload with N=1,000 participants, M=1,000 participations, and O=100 payouts.

> ![](results/scaling.png)
>
> **Figure 2:** Plot of measured median execution times in seconds of the participation protocol based on a synthetic workload with either <span style="color: #a32638;">qualifier (‒, top)</span> or <span style="color: #26547c;">disqualifier (✖, bottom)</span> varied from n ∈ [0..12] and the other pinned to 0.
> Vertical range indicates the min/max execution times.

### Reproducing Results
The Popper workflow in this repository can be used to replicate results, compute statistics, and generate a box plot from the evaluation results.
Assuming both [Docker](https://www.docker.com/) and the [Popper CLI tool](https://getpopper.io/) are installed, you can simply call `popper run` in the root directory of the repository to run the workflow.

Our Popper workflow (see [`.popper.yml`](.popper.yml)) consists of two steps:
 * `measure` - Uses puppeteer to log the time it takes to execute the synthetic workload of PrePaMS operations.
 * `analyze` - Computes basic statistics on the time data collected in the previous step and generates a box plot.

Post execution the [`evaluation/results/`](results) directory will contain the following files:
 * `performance/register.csv` - Measurements of the register protocol.
 * `performance/participations.csv` - Measurements of the participation protocol.
 * `performance/payout.csv` - Measurements of the payout protocol.
 * `plot.pdf` - A violin plot showing the processing times of the measured algorithms.
 * `scaling.pdf` - A min/max linechart showing the processing time of the participation protocol in relation to the number of qualifier and disqualifier.

## License
The PrePaMS prototype and related artifacts is licensed under the terms of the [MIT license](LICENSE).