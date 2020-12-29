FROM ubuntu:latest

### base ###
RUN apt-get update -q \
    && DEBIAN_FRONTEND=noninteractive apt-get install -yq \
        zip \
        unzip \
        bash-completion \
        build-essential \
        htop \
        jq \
        less \
        locales \
        man-db \
        nano \
        software-properties-common \
        sudo \
        time \
        vim \
        curl \
    && locale-gen en_US.UTF-8 \
    && apt-get clean && rm -rf /var/lib/apt/lists/* /tmp/*

ENV LANG=en_US.UTF-8
ENV TZ=Europe/Paris

RUN add-apt-repository -y ppa:git-core/ppa \
    && apt-get install -yq git \
    && rm -rf /var/lib/apt/lists/*

RUN useradd -l -u 33333 -G sudo -md /home/gitpod -s /bin/bash -p gitpod gitpod \
    # passwordless sudo for users in the 'sudo' group
    && sed -i.bkp -e 's/%sudo\s\+ALL=(ALL\(:ALL\)\?)\s\+ALL/%sudo ALL=NOPASSWD:ALL/g' /etc/sudoers
ENV HOME=/home/gitpod
WORKDIR $HOME

USER gitpod

RUN git config --global alias.st 'status -s' \
    && git config --global alias.rs 'reset --soft' \
    && git config --global alias.rh 'reset --hard'

RUN curl -fsSL https://sh.rustup.rs | sh -s -- -y --profile minimal --default-toolchain beta \
    && .cargo/bin/rustup component add \
        rls \
        rust-analysis \
        rust-src \
        rustfmt \
    && .cargo/bin/rustup completions bash | sudo tee /etc/bash_completion.d/rustup.bash-completion > /dev/null \
    && .cargo/bin/rustup completions bash cargo | sudo tee /etc/bash_completion.d/rustup.cargo-bash-completion > /dev/null

RUN bash -lc "cargo install cargo-watch cargo-edit cargo-tree"
