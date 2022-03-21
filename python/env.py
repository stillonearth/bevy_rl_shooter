import subprocess
import imageio
import time
import requests
import numpy as np

from PIL import Image, ImageOps


API_SCREEN = 'http://127.0.0.1:7878/screen.png'
API_STEP = 'http://127.0.0.1:7878/step'

class Environment:

    def __init__(self, executable_path, size):
        self.executable_path = executable_path
        self.size = size
        self.__start()
        
    def reset(self):
        self.rs_env.kill()
        self.__start()

    def __start(self):
        self.rs_env = subprocess.Popen([self.executable_path, "--mode", "train"])
        time.sleep(0.1)
        
    def step(self, action):
        response = requests.post(API_STEP, action, timeout=5)
        return (response.json(), self.visual_observations())

    def visual_observations(self):
        image = imageio.imread(API_SCREEN)
        image = Image.fromarray(image.astype('uint8'), 'RGBA')
        image.thumbnail((self.size[0], self.size[1]))

        return np.asarray(ImageOps.grayscale(image))