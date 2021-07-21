initSidebarItems({"struct":[["CSIDriver","CSIDriver captures information about a Container Storage Interface (CSI) volume driver deployed on the cluster. Kubernetes attach detach controller uses this object to determine whether attach is required. Kubelet uses this object to determine whether pod information needs to be passed on mount. CSIDriver objects are non-namespaced."],["CSIDriverSpec","CSIDriverSpec is the specification of a CSIDriver."],["CSINode","CSINode holds information about all CSI drivers installed on a node. CSI drivers do not need to create the CSINode object directly. As long as they use the node-driver-registrar sidecar container, the kubelet will automatically populate the CSINode object for the CSI driver as part of kubelet plugin registration. CSINode has the same name as a node. If the object is missing, it means either there are no CSI Drivers available on the node, or the Kubelet version is low enough that it doesn’t create this object. CSINode has an OwnerReference that points to the corresponding node object."],["CSINodeDriver","CSINodeDriver holds information about the specification of one CSI driver installed on a node"],["CSINodeSpec","CSINodeSpec holds information about the specification of all CSI drivers installed on a node"],["StorageClass","StorageClass describes the parameters for a class of storage for which PersistentVolumes can be dynamically provisioned."],["VolumeAttachment","VolumeAttachment captures the intent to attach or detach the specified volume to/from the specified node."],["VolumeAttachmentSource","VolumeAttachmentSource represents a volume that should be attached. Right now only PersistenVolumes can be attached via external attacher, in future we may allow also inline volumes in pods. Exactly one member can be set."],["VolumeAttachmentSpec","VolumeAttachmentSpec is the specification of a VolumeAttachment request."],["VolumeAttachmentStatus","VolumeAttachmentStatus is the status of a VolumeAttachment request."],["VolumeError","VolumeError captures an error encountered during a volume operation."],["VolumeNodeResources","VolumeNodeResources is a set of resource limits for scheduling of volumes."]]});