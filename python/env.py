import imageio
import requests
import numpy as np
import json 

from PIL import Image, ImageOps


API_SCREEN = 'http://127.0.0.1:7878/screen.png'
API_STEP = 'http://127.0.0.1:7878/step'
API_RESET = 'http://127.0.0.1:7878/reset'

class Environment:

    def __init__(self, executable_path, size, number_of_agents):
        self.executable_path = executable_path
        self.size = size
        self.number_of_agents = number_of_agents
        
    def reset(self):
        requests.post(API_RESET)
        return
        
    def step(self, actions):
        actions = [{"action": a} for a in actions]
        action_json = json.dumps(actions, indent = 4)
        response = requests.post(API_STEP, action_json)
        return (response.json(), self.visual_observations())

    def visual_observations(self):
        image = imageio.imread(API_SCREEN)
        image = Image.fromarray(image.astype('uint8'), 'RGBA')

        images = []
        for n in range(0, self.number_of_agents):
            crop_rectangle = (
                n*(image.width/self.number_of_agents), 
                0, 
                (n+1)*(image.width/self.number_of_agents), 
                image.height,
            )
    
            cropped_image = image.crop(crop_rectangle)
            cropped_image.thumbnail(self.size)
            images.append(np.asarray(ImageOps.grayscale(cropped_image)))

        return images