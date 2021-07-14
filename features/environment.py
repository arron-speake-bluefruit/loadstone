import shutil
import os

def before_all(context):
    shutil.copy("default_config.ron", "scenario_config.ron")

def after_all(context):
    os.remove("scenario_config.ron")
