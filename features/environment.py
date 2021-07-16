import shutil
import os

def before_all(context):
    with open(".devkit_state", "w") as devkit_state:
        devkit_state.write("dirty")

def before_scenario(context, scenario):
    shutil.copy("default_config.ron", ".scenario_config.ron")

def after_scenario(context, scenario):
    os.remove(".scenario_config.ron")

def after_all(context):
    os.remove(".devkit_state")
