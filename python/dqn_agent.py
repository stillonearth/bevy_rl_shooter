import numpy as np
import random

from collections import namedtuple, deque
from recordtype import recordtype

import torch
import torch.nn.functional as F
import torch.optim as optim

from model import QNetwork

BUFFER_SIZE = int(1e4)  # replay buffer size
BATCH_SIZE = 256        # minibatch size
GAMMA = 0.99            # discount factor
TAU = 1e-3              # for soft update of target parameters
LR = 5e-4               # learning rate
UPDATE_EVERY = 4        # how often to update the network

device = torch.device("cuda:0" if torch.cuda.is_available() else "cpu")


class Agent():
    """Interacts with and learns from the environment."""

    def __init__(self, state_size, action_size, seed, double=False, priority_replay=False, q_network=QNetwork):
        """Initialize an Agent object.
        Params
        ======
            state_size (int): dimension of each state
            action_size (int): dimension of each action
            seed (int): random seed
        """
        self.state_size = state_size
        self.action_size = action_size
        self.seed = random.seed(seed)

        # Double QNS
        self.double = double
        self.batch_indices = torch.arange(0, BATCH_SIZE)

        self.qnetwork_local = q_network(
            state_size=state_size, 
            action_size=action_size, 
            seed=seed).to(device)
        self.qnetwork_target = q_network(
            state_size=state_size, 
            action_size=action_size, 
            seed=seed).to(device)

        self.optimizer = optim.Adam(self.qnetwork_local.parameters(), lr=LR)

        # Replay memory
        self.priority_replay = priority_replay
        if not self.priority_replay:
            self.memory = ReplayBuffer(
                action_size, BUFFER_SIZE, BATCH_SIZE, seed)
        else:
            self.memory = PrioritizedReplayBuffer(
                action_size, BUFFER_SIZE, BATCH_SIZE, seed)

        # Initialize time step (for updating every UPDATE_EVERY steps)
        self.t_step = 0

    def step(self, state, action, reward, next_state, done):
        # Save experience in replay memory
        self.memory.add(state, action, reward, next_state, done)

        # Learn every UPDATE_EVERY time steps.
        self.t_step = (self.t_step + 1) % UPDATE_EVERY
        if self.t_step == 0:
            # If enough samples are available in memory, get random subset and learn
            if len(self.memory) > BATCH_SIZE:
                experiences, idxs = self.memory.sample()
                self.learn(experiences, GAMMA, idxs)

    def act(self, state, eps=0.):
        """Returns actions for given state as per current policy.
        Params
        ======
            state (array_like): current state
            eps (float): epsilon, for epsilon-greedy action selection
        """

        state = torch.from_numpy(state).float().unsqueeze(0).to(device)
        self.qnetwork_local.eval()
        with torch.no_grad():
            action_values = self.qnetwork_local(state)
        self.qnetwork_local.train()

        # Epsilon-greedy action selection
        if random.random() > eps:
            return np.argmax(action_values.cpu().data.numpy())
        else:
            return random.choice(np.arange(self.action_size))

    def learn(self, experiences, gamma, idxs):
        """Update value parameters using given batch of experience tuples.
        Params
        ======
            experiences (Tuple[torch.Tensor]): tuple of (s, a, r, s', done) tuples 
            gamma (float): discount factor
        """
        states, actions, rewards, next_states, dones = experiences

        "*** YOUR CODE HERE ***"

        # Vanilla DQN
        if not self.double:
            Q_targets_next = self.qnetwork_target.forward(
                next_states).detach().max(1)[0].unsqueeze(1)

        # Double DQN
        if self.double:
            action_indices = self.qnetwork_local.forward(
                next_states).detach().argmax(1)
            q_next = self.qnetwork_target.forward(next_states).detach()
            Q_targets_next = q_next[self.batch_indices,
                                    action_indices].view(BATCH_SIZE, 1)

        Q_targets = rewards + gamma * Q_targets_next * (1 - dones)
        Q_expected = self.qnetwork_local(states).gather(1, actions)

        loss = F.mse_loss(Q_expected, Q_targets)

        # Prioritized Experience Replay
        if self.priority_replay:
            td_error = (
                Q_expected - Q_targets).detach().abs().cpu().numpy().reshape(-1)
            self.memory.update_priorities(idxs, td_error)
            p = self.memory.get_probabilities_from_indices(idxs)
            p = torch.cuda.FloatTensor((1. / BATCH_SIZE) * (1. / p))
            loss = (p * loss).mean()

        self.optimizer.zero_grad()
        loss.backward()
        self.optimizer.step()

        # ------------------- update target network ------------------- #
        self.soft_update(self.qnetwork_local, self.qnetwork_target, TAU)

    def soft_update(self, local_model, target_model, tau):
        """Soft update model parameters.
        θ_target = τ*θ_local + (1 - τ)*θ_target
        Params
        ======
            local_model (PyTorch model): weights will be copied from
            target_model (PyTorch model): weights will be copied to
            tau (float): interpolation parameter 
        """
        for target_param, local_param in zip(target_model.parameters(), local_model.parameters()):
            target_param.data.copy_(
                tau*local_param.data + (1.0-tau)*target_param.data)


class ReplayBuffer:
    """Fixed-size buffer to store experience tuples."""

    def __init__(self, action_size, buffer_size, batch_size, seed):
        """Initialize a ReplayBuffer object.
        Params
        ======
            action_size (int): dimension of each action
            buffer_size (int): maximum size of buffer
            batch_size (int): size of each training batch
            seed (int): random seed
        """
        self.action_size = action_size
        self.memory = deque(maxlen=buffer_size)
        self.batch_size = batch_size
        self.experience = namedtuple("Experience", field_names=[
                                     "state", "action", "reward", "next_state", "done"])
        self.seed = random.seed(seed)

    def add(self, state, action, reward, next_state, done):
        """Add a new experience to memory."""
        e = self.experience(state, action, reward, next_state, done)
        self.memory.append(e)

    def sample(self):
        """Randomly sample a batch of experiences from memory."""
        experiences = random.sample(self.memory, k=self.batch_size)

        states = torch.from_numpy(
            np.vstack([e.state for e in experiences if e is not None])).float().to(device)
        actions = torch.from_numpy(
            np.vstack([e.action for e in experiences if e is not None])).long().to(device)
        rewards = torch.from_numpy(
            np.vstack([e.reward for e in experiences if e is not None])).float().to(device)
        next_states = torch.from_numpy(np.vstack(
            [e.next_state for e in experiences if e is not None])).float().to(device)
        dones = torch.from_numpy(np.vstack(
            [e.done for e in experiences if e is not None]).astype(np.uint8)).float().to(device)

        return (states, actions, rewards, next_states, dones), None

    def __len__(self):
        """Return the current size of internal memory."""
        return len(self.memory)


class PrioritizedReplayBuffer:

    def __init__(self, action_size, buffer_size, batch_size, seed, epsilon=0.0001, alpha=0.9):
        self.action_size = action_size
        self.memory = deque(maxlen=buffer_size)
        self.batch_size = batch_size
        self.experience = recordtype("Experience", field_names=[
            "state",
            "action",
            "reward",
            "next_state",
            "done",
            "priority",
            "probability",
        ])
        self.seed = random.seed(seed)
        self.epsilon = epsilon
        self.alpha = alpha

    def add(self, state, action, reward, next_state, done):
        e = self.experience(state, action, reward,
                            next_state, done, 1 / self.epsilon, 0)
        self.memory.append(e)

    def get_probabilities_from_priorities(self):
        priorities = np.array(
            [e.priority for e in self.memory if e is not None])
        scaled_priorities = (priorities + self.epsilon)**self.alpha
        return scaled_priorities / np.sum(scaled_priorities)

    def get_probabilities_from_indices(self, idx):
        return np.array([self.memory[i].probability for i in idx])

    def sample(self):
        probabilities = self.get_probabilities_from_priorities()
        idxs = np.random.choice(
            np.arange(0, len(self.memory)), self.batch_size, p=probabilities)
        experiences = []
        for j, i in enumerate(idxs):
            self.memory[i].probability = probabilities[j]
            experiences.append(self.memory[i])

        states = torch.from_numpy(
            np.vstack([e.state for e in experiences if e is not None])).float().to(device)
        actions = torch.from_numpy(
            np.vstack([e.action for e in experiences if e is not None])).long().to(device)
        rewards = torch.from_numpy(
            np.vstack([e.reward for e in experiences if e is not None])).float().to(device)
        next_states = torch.from_numpy(np.vstack(
            [e.next_state for e in experiences if e is not None])).float().to(device)
        dones = torch.from_numpy(np.vstack(
            [e.done for e in experiences if e is not None]).astype(np.uint8)).float().to(device)

        return (states, actions, rewards, next_states, dones), idxs

    def update_priorities(self, idxs, weights):
        for (i, w) in zip(idxs, weights):
            self.memory[i].priority = w

    def __len__(self):
        return len(self.memory)