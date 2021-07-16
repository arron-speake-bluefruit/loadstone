#!/usr/bin/env python3

import os
from typing import Optional, Mapping
import subprocess
from behave import *

SCENARIO_CONFIG_PATH = ".scenario_config.ron"

DEVKIT_STATE_CLEAN = 1
DEVKIT_STATE_CLEAN_EXTERNAL = 2
DEVKIT_STATE_DIRTY = 3
devkit_state = DEVKIT_STATE_DIRTY

def start_process(*args: str, environment: Optional[Mapping[str, str]] = None) -> subprocess.Popen:
    """Starts a new process with the provided arguments and environment."""
    return subprocess.Popen(list(args), env = environment)

def end_process(process: subprocess.Popen):
    """Waits for a process to exit, asserting that it succeeded."""
    exit_code = process.wait()
    assert exit_code == 0

def modify_config(*args: str):
    """Modifies the loadstone config file using confedit and the provided arguments."""

    arguments = [ "cargo", "run", "--manifest-path=loadstone/tools/confedit/Cargo.toml", "--" ]
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

def start_cleanup() -> Optional[subprocess.Popen]:
    """Starts a subprocess to format the devkit, depending on the devkit state."""
    if devkit_state == DEVKIT_STATE_CLEAN:
        return None
    elif devkit_state == DEVKIT_STATE_CLEAN_EXTERNAL:
        return start_process("scripts/format", "--internal-only")
    else: # devkit_state == DEVKIT_STATE_DIRTY
        return start_process("scripts/format")

def end_cleanup(process: Optional[subprocess.Popen]):
    """Wait for the format to finish and mark the devkit as clean."""
    if process != None:
        end_process(process)
    devkit_state = DEVKIT_STATE_CLEAN

@given("loadstone is configured for custom greeting \"{greeting}\"")
def step_impl(context, greeting):
    modify_config("--greeting", greeting)

@given("loadstone is configured for a golden bank")
def step_impl(context):
    modify_config("--golden", "2")

@given("loadstone is not configured for a golden bank")
def step_impl(context):
    modify_config("--golden", "none")

@given("loadstone is configured for serial recovery")
def step_impl(context):
    modify_config("--recovery", "true")

@given("just loadstone is loaded on the devkit")
def step_impl(context):
    environment = os.environ.copy()
    environment["LOADSTONE_CONFIG"] = read_scenario_config()
    build_process = start_process("scripts/build", environment = environment)

    cleanup = start_cleanup()

    end_process(build_process)
    end_cleanup(cleanup)

    flash_process = start_process("st-flash", "write", "loadstone.bin", "0x08000000")
    end_process(flash_process)
    devkit_state = DEVKIT_STATE_CLEAN_EXTERNAL

@when("the devkit is powered on")
def step_impl(context):
    process = start_process("scripts/reset-usb")
    end_process(process)

@then("the following is printed to the cli")
def step_impl(context):
    SERIAL_DEVICE_PATH = "/dev/ttyUSB0"
    with open(SERIAL_DEVICE_PATH, "rb") as file:
        for expected in bytes(context.text, "utf-8"):
            actual = file.read(1)
            assert len(actual) == 1
            assert expected == actual[0]
