from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from typing import ClassVar, Optional

DESCRIPTOR: _descriptor.FileDescriptor

class Scalar(_message.Message):
    __slots__ = ["data"]
    DATA_FIELD_NUMBER: ClassVar[int]
    data: bytes
    def __init__(self, data: Optional[bytes] = ...) -> None: ...
