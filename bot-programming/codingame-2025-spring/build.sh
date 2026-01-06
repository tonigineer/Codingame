#!/bin/bash
# Build the Maven project using Docker

docker run --rm --name maven-builder -v "$(pwd)":/app -w /app maven:3.9-eclipse-temurin-17 mvn clean package
