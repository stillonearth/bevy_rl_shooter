import torch
import torch.nn as nn
import torch.nn.functional as F
import torchvision


class QNetwork(nn.Module):
    """Actor (Policy) Model."""

    def __init__(self, state_size, action_size, seed, fc1_units=64, fc2_units=64):
        """Initialize parameters and build model.
        Params
        ======
            state_size (int): Dimension of each state
            action_size (int): Dimension of each action
            seed (int): Random seed
            fc1_units (int): Number of nodes in first hidden layer
            fc2_units (int): Number of nodes in second hidden layer
        """
        super(QNetwork, self).__init__()
        self.seed = torch.manual_seed(seed)
        self.fc1 = nn.Linear(state_size, fc1_units)
        self.fc2 = nn.Linear(fc1_units, fc2_units)
        self.fc3 = nn.Linear(fc2_units, action_size)

    def forward(self, state):
        """Build a network that maps state -> action values."""
        x = F.relu(self.fc1(state))
        x = F.relu(self.fc2(x))
        return self.fc3(x)


class DuelingQNetwork(nn.Module):
    
    def __init__(self, state_size, action_size, seed, fc1_units=256, fc2_units=256, fc_a_units=128, fc_v_units=128):
        super(DuelingQNetwork, self).__init__()
        self.seed = torch.manual_seed(seed)

        self.fc1 = nn.Linear(state_size, fc1_units)
        self.fc2 = nn.Linear(fc1_units, fc2_units)

        self.fc_h_a = nn.Linear(fc2_units, fc_a_units)
        self.fc_z_a = nn.Linear(fc_a_units, action_size)

        self.fc_h_v = nn.Linear(fc2_units, fc_v_units)
        self.fc_z_v = nn.Linear(fc_v_units, 1)

    def forward(self, state):
        x = F.relu(self.fc1(state))
        x = F.relu(self.fc2(x))

        v = F.relu(self.fc_h_v(x))
        v = self.fc_z_v(v)

        a = F.relu(self.fc_h_a(x))
        a = self.fc_z_a(a)

        q = v + a - a.mean()
        return q


class VisualQNetwork(nn.Module):

    def __init__(self, state_size, action_size, seed, fc1_units=32, fc2_units=64):
        super(VisualQNetwork, self).__init__()
        self.conv1 = nn.Conv2d(3, 16, kernel_size=5, stride=2)
        self.bn1 = nn.BatchNorm2d(16)
        self.conv2 = nn.Conv2d(16, 32, kernel_size=5, stride=2)
        self.bn2 = nn.BatchNorm2d(32)
        self.conv3 = nn.Conv2d(32, 32, kernel_size=5, stride=2)
        self.bn3 = nn.BatchNorm2d(32)

        self.w = state_size[0]
        self.h = state_size[1]
        self.n_channels = state_size[2]

        # Number of Linear input connections depends on output of conv2d layers
        # and therefore the input image size, so compute it.
        def conv2d_size_out(size, kernel_size = 5, stride = 2):
            return ((size - (kernel_size - 1) - 1) // stride  + 1)
        convw = conv2d_size_out(conv2d_size_out(conv2d_size_out(self.w)))
        convh = conv2d_size_out(conv2d_size_out(conv2d_size_out(self.h)))
        linear_input_size = convw * convh * 32 
        self.fc1 = nn.Linear(linear_input_size, fc1_units)
        self.fc2 = nn.Linear(fc1_units, fc2_units)
        self.fc3 = nn.Linear(fc2_units, action_size)
    
    def forward(self, x):

        if len(x.shape) != 4:
            x = x.view(-1, 3, self.w, self.h)

        x = F.relu(self.bn1(self.conv1(x)))
        x = F.relu(self.bn2(self.conv2(x)))
        x = F.relu(self.bn3(self.conv3(x)))
        
        x = F.relu(self.fc1(x.view(x.size(0), -1)))
        x = F.relu(self.fc2(x))
        return self.fc3(x)