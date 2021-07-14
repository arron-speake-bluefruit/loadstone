#!/usr/bin/env python3

import os
from typing import Optional, Mapping
import subprocess
import typing
from behave import *

SCENARIO_CONFIG_PATH = "./scenario_config.ron"

def start_process(*args: str, environment: Optional[Mapping[str, str]] = None):
    """Starts a new process with the provided arguments and environment."""
    return subprocess.Popen(list(args), env = environment, shell = True)

def end_process(process: subprocess.Popen):
    """Waits for a process to exit, asserting that it succeeded."""
    exit_code = process.wait()
    assert exit_code == 0

def modify_config(*args: str):
    """Modifies the loadstone config file using config-generator and the provided arguments."""
    CONFIG_GENERATOR_PATH = "./config-generator"

    arguments = [ CONFIG_GENERATOR_PATH ]
    arguments.extend(args)

    with open(SCENARIO_CONFIG_PATH, "rb") as file:
        input_config = file.read()

    process = subprocess.Popen(arguments, stdin=subprocess.PIPE, stdout=subprocess.PIPE);
    output_config = process.communicate(input_config)[0]
    exit_code = process.wait()
    assert exit_code == 0

    with open(SCENARIO_CONFIG_PATH, "wb") as file:
        file.write(output_config)

def read_scenario_config() -> str:
    """Returns the loadstone configuration for the active scenario."""
    with open(SCENARIO_CONFIG_PATH, "r") as file:
        return file.read()

@given("loadstone is configured for custom greeting \"{greeting}\"")
def step_impl(context, greeting):
    modify_config("--greeting", greeting)

@given("loadstone is configured for a golden bank")
def step_impl(context):
    # NOTE: This assumes that loadstone is currently configured with a bank 3.
    modify_config("--golden", "2")

@given("loadstone is configured for serial recovery")
def step_impl(context):
    modify_config("--recovery", "true")

@given("just loadstone is loaded on the devkit")
def step_impl(context):
    environment = os.environ.copy()
    environment["LOADSTONE_CONFIG"] = read_scenario_config()
    build_process = start_process("scripts/build", environment = environment)

    format_process = start_process("scripts/format/run")

    end_process(build_process)
    end_process(format_process)

    flash_process = start_process("st-flash", "write", "loadstone.bin", "0x08000000")
    end_process(flash_process)

@when("the devkit is powered on")
def step_impl(context):
    process = start_process("scripts/format/reset_usb")
    end_process(process)

@then("the following is printed to the cli")
def step_impl(context):
    SERIAL_DEVICE_PATH = "/dev/ttyUSB0"
    with open(SERIAL_DEVICE_PATH, "rb") as file:
        for expected in bytes(context.text, "utf-8"):
            actual = file.read(1)
            assert len(actual) == 1
            assert expected == actual[0]
