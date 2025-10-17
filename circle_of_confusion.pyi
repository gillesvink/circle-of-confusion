from typing import Tuple
from enum import IntEnum

class Math(IntEnum):
    """Math mode of provided depth channel

    This enum defines how the depth channel's mathematical representation
    should be interpreted. The mode is specified by the render engine that
    generates the depth information."""

    REAL = 1
    """Real depth values"""
    OneDividedByZ = 2
    """One divided by Z (inverse depth)"""

class WorldUnit(IntEnum):
    """World unit that specifies the unit for the depth channel

    This enum defines the units that can be used for depth channel measurements.
    The unit determines how depth values are interpreted in the rendering process."""

    MM = 1
    """Millimeters"""
    CM = 2
    """Centimeters"""
    DM = 3
    """Decimeters"""
    M = 4
    """Meters"""
    INCH = 5
    """Inches"""
    FT = 6
    """Feet"""

class CameraData:
    """
    Camera parameters that are used when using the camera-based circle of confusion

    This struct contains camera-specific parameters that are used to calculate
    circle of confusion.

    Args:
        focal_length: Focal length of the camera lens
        f_stop: F-stop (aperture) of the camera
        filmback: Filmback size (width and height)
        near_field: Near field distance for depth of field
        far_field: Far field distance for depth of field
        world_unit: World unit for depth measurements
        resolution: Resolution camera is recording at

    """
    def __new__(
        self,
        focal_length: float,
        f_stop: float,
        filmback: Tuple[float, float],
        near_field: float,
        far_field: float,
        world_unit: float,
        resolution: Tuple[int, int],
    ) -> "Settings": ...

class Settings:
    """
    Settings for the circle of confusion calculation.

    This struct contains all the parameters needed to calculate the circle of confusion.

    Args:
        size: Base size for circle of confusion
        max_size: Maximum size for circle of confusion (clamped)
        math: Mathematical interpretation of depth
        focal_plane: Focal plane position
        protect: Protection value for circle of confusion, this adds a region that has a CoC of zero.
        pixel_aspect: Pixel aspect ratio
        camera_data: Camera parameters

    """
    def __new__(
        self,
        size: float,
        max_size: float,
        math: Math,
        focal_plane: float,
        protect: float,
        pixel_aspect: float,
        camera_data: CameraData = None,
    ) -> "Settings": ...

class Calculator:
    """Calculator that is able to calculate the Circle of Confusion based on the provided settings.

    The size in px is the radius of the convolution.
    A CoC of 10 would mean a diameter of 20 pixels.

    ## Output:
    * `+` is for far field pixels
    * `-` is for near field pixels

    ## Modes+
    The calculator supports two modes, one is physically accurate,
    the other lets you tune your own DoF size.

    ### Manually
    When no camera data is provided to the Settings struct, the manual mode will be used.
    This creates a smooth falloff to the focal plane point, based on the size and max size added.

    It will gradually apply the size based on the distance from the focal plane.
    When using a larger size, the CoC will be increased.

    Protect can be used to apply a safe region to the focal plane.

    ### Camera
    When the camera data is applied to the settings,
    the camera values will be used instead. This matches real world CoC values.

    Lowering f-stop will increase the CoC values, just like increasing the focal-length would.

    This calculation is based on the CoC algorithm:
    [Wikipedia](https:en.wikipedia.org/wiki/Circle_of_confusion)

    ## Usage
    Use the `Calculator(settings)` method to create the instance
    and let the necessary parameters be pre-computed.

    To update the settings, for example when changing the focal plane,
    call the `update_settings` method.

    ## Example
    >>> from circle_of_confusion import Calculator, Settings, Math, CameraData, WorldUnit

    >>> camera_data = CameraData(
    ...     focal_length=100.0,
    ...     f_stop=2.0,
    ...     filmback=(24.576, 18.672),
    ...     near_field=0.1,
    ...     far_field=10000.0,
    ...     world_unit=WorldUnit.M,
    ...     resolution=(1920, 1080),
    ... )
    >>> settings = Settings(
    ...     size=10,
    ...     max_size=100,
    ...     math=Math.REAL,
    ...     focal_plane=30.0,
    ...     protect=0.0,
    ...     pixel_aspect=1.0,
    ...     camera_data=camera_data,
    ... )
    >>> calculator = Calculator(settings)
    >>> calculator.calculate(10)
    -11.93532943725586
    """

    def __new__(self, settings: Settings) -> "Calculator":
        """Create a new instance of the Calculator with the specified settings.

        Automatically calculates all necessary values for calculations.
        """
        ...
    def update_settings(self, settings: Settings) -> None:
        """Update the instance of the Calculator with the specified settings.

        Automatically calculates all necessary values for calculations.
        """
        ...

    def calculate(self, value: float) -> float:
        """Perform calculation based on the input value.

        This can be called upon a depth map for each pixel for example,
        to calculate the size of convolution for each pixel.

        Args:
            value:  the input value in depth to get the circle of
                    confusion value from.
        """
        ...
