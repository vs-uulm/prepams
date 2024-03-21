# PrePaMS: A Privacy-preserving Participant Management System

This repository contains source code artifacts, experiments, and results associated with our PETS'24 submission.
A publicly hosted test deployment can be accessed at http://j2cqrb6vkmz32gqjn3jvjge5rm64agwpykjgvnp2yo372voytrcehyad.onion/ (blinded for review and therefore only accessible via Tor), where anyone is able to explore our system using any device featuring a modern web browser.

## Repository Structure
 * [`shared/`](shared) - a rust-based webassembly module implementing the PrePaMS schema.
 * [`backend/`](backend) - the PrePaMS server application that interfaces with the PrePaMS web application.
 * [`frontend/`](frontend) - the main client-side source code of the PrePaMS web application.
 * [`evaluation/`](evaluation) - evaluation artifacts to reproduce our performance evluation results (see [Evaluation](#evaluation)).
 * [`results/`](results) - our evaluation results as reported in our submission.

## Abstract
> Taking part in surveys, experiments, and studies is often compensated by rewards to increase the number of participants and encourage attendance.
> While privacy requirements are usually considered for participation, privacy aspects of the reward procedure are mostly ignored.
> To this end, we introduce PrePaMS, an efficient participation management system that supports prerequisite checks and participation rewards in a privacy-preserving way.
> Our system organizes participations with potential (dis-)qualifying dependencies and enables secure reward payoffs.
> By leveraging a set of proven cryptographic primitives and mechanisms such as anonymous credentials and zero-knowledge proofs, participations are protected so that service providers and organizers cannot derive the identity of participants even within the reward process.
> In this paper, we have designed and implemented a prototype of PrePaMS to show its effectiveness and evaluated its performance under realistic workloads.
> PrePaMS covers the information whether subjects have participated in surveys, experiments, or studies.
> When combined with other secure solutions for the actual data collection within these events, PrePaMS can represent a cornerstone for more privacy-preserving empirical research.
>
> ![](results/overview.png)


## Evaluation
To assess the practicability of our proof-of-concept prototype, we have evaluated the performance of the PrePaMS primitives.
The evaluation in this repository follows the [Popper convention](https://getpopper.io/) for reproducible evaluations.

> ![](results/plot.png)
> **Figure 1:** Combined violin/jitter plots of measured execution
times (in seconds) of our PrePaMS proof of concept implementation based on a synthetic workload with 𝑁 = 1000 participants, 𝑀 = 1000 participations, and 𝑂 = 100 payouts.
> The individual protocols are segmented by the role of the executing party (P: Participant, O: Organizer, S Service) and partially replicated across different device types (i.e., 
> <img src="data:image/svg+xml,%0A%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24'%3E%3Ctitle%3Efa-laptop%3C/title%3E%3Cpath d='m5.2 19.175q-.825 0-1.4125-.5875t-.5875-1.4125v-8.8q0-.825.5875-1.4125t1.4125-.5875h13.6q.825 0 1.4125.5875t.5875 1.4125v8.8q0 .825-.5875 1.4125t-1.4125.5875h-13.6zm-.4-10.8v8.8q0 .1625.1187.2813t.2813.1188h13.6q.1625 0 .2813-.1188t.1188-.2813v-8.8q0-.1625-.1188-.2813t-.2813-.1187h-13.6q-.1625 0-.2813.1187t-.1187.2813zm17.2 11.6h2v1.2q0 .5-.5875.85t-1.4125.35h-20q-.825 0-1.4125-.35t-.5875-.85v-1.2h2 20zm-9 1.2q.2 0 .2-.2t-.2-.2h-2q-.2 0-.2.2t.2.2h2z' /%3E%3C/svg%3E" alt="" width="16" height="16"> desktop,
> <img src="data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24'%3E%3Ctitle%3Efa-tablet%3C/title%3E%3Cpath d='m12.27 21.82q0-.44-.32-.77t-.77-.32-.77.32-.32.77.32.77.77.32.77-.32.32-.77zm6.55-2.73v-16.36q0-.22-.16-.38t-.38-.16h-14.18q-.22 0-.38.16t-.16.38v16.36q0 .22.16.38t.38.16h14.18q.22 0 .38-.16t.16-.38zm2.18-16.36v18.55q0 1.13-.8 1.93t-1.93.8h-14.18q-1.13 0-1.93-.8t-.8-1.93v-18.55q0-1.13.8-1.93t1.93-.8h14.18q1.13 0 1.93.8t.8 1.93z' /%3E%3C/svg%3E" alt="" width="16" height="16"> tablet,
> and <img src="data:image/svg+xml,%0A%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24'%3E%3Ctitle%3Efa-mobile%3C/title%3E%3Cpath d='m13.7 21.6q0-.62-.44-1.06t-1.06-.44-1.06.44-.44 1.06.44 1.06 1.06.44 1.06-.44.44-1.06zm3.9-3v-13.2q0-.24-.18-.42t-.42-.18h-9.6q-.24 0-.42.18t-.18.42v13.2q0 .24.18.42t.42.18h9.6q.24 0 .42-.18t.18-.42zm-3.6-15.9q0-.3-.3-.3h-3q-.3 0-.3.3t.3.3h3q.3 0 .3-.3zm5.4-.3v19.2q0 .97-.71 1.69t-1.69.71h-9.6q-.98 0-1.69-.71t-.71-1.69v-19.2q0-.97.71-1.69t1.69-.71h9.41q1.17 0 1.88.71t.71 1.69z' /%3E%3C/svg%3E" alt="" width="16" height="16"> smartphone).


> ![](results/scaling.png)
> **Figure 2:** Plot of measured median execution times in seconds of the participation protocol based on a synthetic workload with either <span style="color: #a32638;">qualifier (×, 𝑞)</span>, <span style="color: #26547c;">disqualifier (+, 𝑑)</span>, <span style="color: #bd6005;">range constraints (⋄, 𝑟)</span>, or <span style="color: #56aa1c;">set constraints (▽, 𝑠)</span> varied from 𝑛 ∈ [0..10] and all other parameters pinned to 0.

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

### Measuring other Devices

The [`evaluation/`](evaluation/) directory also contains means to measure other devices, such as smartphones or tablets.
Executing `npm run build` compiles a static version of the performance evaluation and outputs it to the `evaluation/dist` directory.
Then [`evaluation/serve.js`](evaluation/serve.js) can be used to run a simple web server which hosts this static evaluation app, as well as an HTTP-based API to retrieve measurement results from other devices.
A ready to use version is deployed under http://j2cqrb6vkmz32gqjn3jvjge5rm64agwpykjgvnp2yo372voytrcehyad.onion/eval/ (blinded for review and hence only accessible through Tor for now), ready to to be used on any device featuring a modern browser.

## License
The PrePaMS prototype and related artifacts is licensed under the terms of the [MIT license](LICENSE).
