import imageio
import requests
import numpy as np
import json

from PIL import Image, ImageOps
from gym import spaces


API_SCREEN = 'http://127.0.0.1:7878/screen.png'
API_STEP = 'http://127.0.0.1:7878/step'
API_RESET = 'http://127.0.0.1:7878/reset'

ACTION_MAP = {
    0: "IDLE",
    1: "TURN_LEFT",
    2: "TURN_RIGHT",
    3: "LEFT",
    4: "RIGHT",
    5: "FORWARD",
    6: "BACKWARD",
    7: "SHOOT",
}


class Environment:

    def __init__(self, executable_path, size, number_of_agents):
        self.executable_path = executable_path
        self.size = size
        self.number_of_agents = number_of_agents
        self.observation_space = spaces.Box(
            low=0, high=255, shape=(self.number_of_agents, size[0], size[1], 1), dtype=np.uint8)
        self.action_space = spaces.Discrete(8)
        self.metadata = {}
        self.images = []

    def reset(self, seed=None):
        requests.post(API_RESET, timeout=10)
        return self.visual_observations(), None

    def step(self, actions):
        actions = [{"action": ACTION_MAP[a]} for a in actions]
        action_json = json.dumps(actions, indent=4)
        response = requests.get(
            API_STEP, params={'payload': action_json}, timeout=10)

        state = response.json()
        observation = self.visual_observations()

        reward = [r['reward'] for r in state]
        terminated = [r['is_terminated'] for r in state]
        truncated = None
        info = None

        return observation, reward, terminated, truncated, info

    def visual_observations(self):
        image = imageio.imread(API_SCREEN)
        image = Image.fromarray(image.astype('uint8'), 'RGBA')

        self.images = []
        for n in range(0, self.number_of_agents):
            crop_rectangle = (
                n*(image.width/self.number_of_agents),
                0,
                (n+1)*(image.width/self.number_of_agents),
                image.height,
            )

            cropped_image = image.crop(crop_rectangle)
            cropped_image.thumbnail(self.size)
            self.images.append(np.asarray(ImageOps.grayscale(cropped_image)))

        return self.images
