# The python package carla 0.9.15 is only available to the Python versions 2.7, 3.7, 3.8, 3.9 and 3.10
FROM python:3.10

WORKDIR /app

COPY automatic_control_zenoh.py requirements.txt /app/

RUN pip install --upgrade pip

RUN pip install --no-cache-dir -r requirements.txt

CMD ["python3", "automatic_control_zenoh.py"]

#Expose port 2000 because of CARLA
#Expose port 7447/tcp and 7447/udp because of zenoh
