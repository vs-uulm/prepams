# syntax=docker/dockerfile:1

FROM rocker/verse:4.2.2
RUN install2.r --error --skipmissing --deps TRUE --skipinstalled tikzDevice cowplot ggrastr
RUN tlmgr install pgf preview grfext fontawesome amsfonts fontspec standalone luatex85

LABEL org.opencontainers.image.ref.name="ghcr.io/vs-uulm/prepams/verse"
LABEL org.opencontainers.image.source="https://github.com/vs-uulm/prepams"
LABEL org.opencontainers.image.authors="Echo Meißner <mail@ech0.de> (https://uni-ulm.de/in/vs/meissner)"
LABEL org.opencontainers.image.licenses="AGPL-3.0-or-later"
