#!/usr/bin/env python3

import os
import time
import subprocess
from typing import Optional, Mapping, BinaryIO
from behave import *

SCENARIO_CONFIG_PATH = ".scenario_config.ron"
DEVKIT_STATE_PATH = ".devkit_state"
SERIAL_DEVICE_PATH = "/dev/ttyUSB0"

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

def assert_valid_devkit_state(state: str):
    assert (state == "clean") or (state == "clean external") or (state == "dirty")

def get_devkit_state() -> str:
    with open(DEVKIT_STATE_PATH, "r") as state_file:
        state = state_file.read()
    assert_valid_devkit_state(state)
    return state

def set_devkit_state(state: str):
    assert_valid_devkit_state(state)
    with open(DEVKIT_STATE_PATH, "w") as state_file:
        return state_file.write(state)

def start_cleanup() -> Optional[subprocess.Popen]:
    """Starts a subprocess to format the devkit, depending on the devkit state."""
    state = get_devkit_state()
    if state == "clean":
        return None
    elif state == "clean external":
        return start_process("scripts/format", "--internal-only")
    else: # state == "dirty"
        return start_process("scripts/format")

def end_cleanup(process: Optional[subprocess.Popen]):
    """Wait for the format to finish and mark the devkit as clean."""
    if process != None:
        end_process(process)
    set_devkit_state("clean")

def try_read_bytes_from_file(file: BinaryIO, expected: bytes, timeout: float) -> bool:
    """Attempts to read `expected` bytes from `file` in `timeout` seconds."""
    start_time = time.time()

    index = 0
    while index < len(expected):
        did_time_out = (time.time() - start_time) > timeout
        if did_time_out:
            return False

        b = file.read(1)
        if len(b) != 1:
            return False

        if b == expected[index]:
            index += 1
        else:
            index = 0
    return True

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
    set_devkit_state("clean external")

@when("the devkit is powered on")
def step_impl(context):
    process = start_process("scripts/reset-usb")
    end_process(process)

@when("loadstone enters serial recovery mode")
def step_impl(context):
    set_devkit_state("dirty")

    EXPECTED = bytes("-- Loadstone Recovery Mode --", "utf-8")
    TIMEOUT_SECONDS = 20.0

    with open(SERIAL_DEVICE_PATH, "rb") as file:
        assert try_read_bytes_from_file(file, EXPECTED, TIMEOUT_SECONDS)

    time.sleep(1)

@when("a golden firmware image is uploaded using xmodem")
def step_impl(context):
    GOLDEN_IMAGE_PATH = "images/golden_demo_app.bin"
    process = start_process("loadstone/tools/upload.sh", GOLDEN_IMAGE_PATH, SERIAL_DEVICE_PATH)
    end_process(process)

@then("the following is printed to the cli")
def step_impl(context):
    with open(SERIAL_DEVICE_PATH, "rb") as file:
        for expected in bytes(context.text, "utf-8"):
            actual = file.read(1)
            assert len(actual) == 1
            assert expected == actual[0]
