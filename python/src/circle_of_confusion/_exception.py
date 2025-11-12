from _circle_of_confusion.circle_of_confusion_pb2 import FFIError


class CircleOfConfusionError(Exception):
    """Exception raised when CoC package fails."""

    @staticmethod
    def map_error(error: int) -> "CircleOfConfusionError":
        """Map the error int to the Error enum by protobuf."""
        if error == FFIError.PROTO_DECODE:
            return CircleOfConfusionError(
                "Protobuf could not be decoded from input buffer",
            )
        if error == FFIError.PROTO_ENCODE:
            return CircleOfConfusionError("Data could not be encoded to protobuf")
        if error == FFIError.INITIALIZE_FAILED:
            return CircleOfConfusionError("Initialization failed")
        return CircleOfConfusionError("Something undefined happened.")
