from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from typing import ClassVar, Iterable, Optional

DESCRIPTOR: _descriptor.FileDescriptor

class Scalar(_message.Message):
    __slots__ = ["data"]
    DATA_FIELD_NUMBER: ClassVar[int]
    data: _containers.RepeatedScalarFieldContainer[bytes]
    def __init__(self, data: Optional[Iterable[bytes]] = ...) -> None: ...
