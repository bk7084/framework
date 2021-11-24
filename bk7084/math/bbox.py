from __future__ import annotations

import math
from dataclasses import dataclass

from ..math import Vec3


@dataclass
class BoundingBox:
    lower: Vec3 = Vec3(math.inf)
    upper: Vec3 = Vec3(-math.inf)

    @classmethod
    def empty(cls) -> BoundingBox:
        return cls()

    @property
    def is_valid(self) -> bool:
        return self.lower == Vec3(math.inf) and self.upper == Vec3(-math.inf)
