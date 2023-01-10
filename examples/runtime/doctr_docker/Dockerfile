FROM ubuntu:20.04
ADD requirements.txt /

# install python and build dependencies for server
RUN apt-get update
RUN apt-get upgrade -y
RUN apt-get install -y gcc \
    gfortran \
    python3-dev \
    python3-pip \
    libopenblas-dev \
    liblapack-dev \
    ffmpeg \
    libsm6 \
    libxext6 \
    python3-opencv \
    libgl1 \ 
    RUN python3 --version
RUN apt-get install -y python3-pip
# RUN apt-get install -y libgtk-3-dev
# RUN apt-get install -y libgeos-dev
# remove old python executable
RUN rm /usr/bin/pip || true
RUN ln -s /usr/bin/pip3 /usr/bin/pip
RUN which python3
RUN rm /usr/bin/python || true
RUN ln -s /usr/bin/python3 /usr/bin/python
# RUN pip3 install -r requirements.txt
RUN pip install "python-doctr[torch]"
RUN pip install opencv-python-headless==4.5.3.56

ENV PYTHONUNBUFFERED 1

# add execution dependencies
ADD equifax_sample_100_pages.pdf /
ADD app.py /
ADD ./models /

WORKDIR "/"
# ENTRYPOINT [ "python", "app.py" ]