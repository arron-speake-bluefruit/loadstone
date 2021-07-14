#!/usr/bin/env python3

import os
from typing import Optional
import subprocess
from behave import *

SCENARIO_CONFIG_PATH = "./scenario_config.ron"

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
    with open(SCENARIO_CONFIG_PATH, "r") as file:
        return file.read()

def find_stmicroelectronics_usb_device() -> Optional[str]:
    """Attempts to find the vendor/product ID of a connected STM32F412 device."""
    process = subprocess.Popen([ "lsusb" ], stdout=subprocess.PIPE)
    devices = process.communicate()[0].decode("utf-8")
    exit_code = process.wait()
    assert exit_code == 0

    lines = devices.splitlines()
    for line in lines:
        if "STMicroelectronics" in line:
            return line[23:32] # Substring of the vendor and product IDs
    return None

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
    # Build loadstone off new configuration.
    environment = os.environ.copy()
    environment["LOADSTONE_CONFIG"] = read_scenario_config()
    cargo_process = subprocess.Popen([ "./build" ], env = environment, shell = True)

    # Format devkit.
    format_process = subprocess.Popen([ "./format/run" ])

    cargo_exit_code = cargo_process.wait()
    format_exit_code = format_process.wait()
    assert (cargo_exit_code == 0) and (format_exit_code == 0)

    # Flash image
    process = subprocess.Popen([ "st-flash", "write", "loadstone.bin", "0x08000000" ])
    exit_code = process.wait()
    assert exit_code == 0

@when("the devkit is powered on")
def step_impl(context):
    vendor_product_id = find_stmicroelectronics_usb_device()
    assert vendor_product_id != None
    process = subprocess.Popen([ "usbreset", vendor_product_id ])
    exit_code = process.wait()
    assert exit_code == 0

@then("the following is printed to the cli")
def step_impl(context):
    SERIAL_DEVICE_PATH = "/dev/ttyUSB0"
    with open(SERIAL_DEVICE_PATH, "rb") as file:
        for expected in bytes(context.text, "utf-8"):
            actual = file.read(1)
            assert len(actual) == 1
            assert expected == actual[0]
