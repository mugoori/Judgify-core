import React from 'react'
import { Handle, Position, NodeProps } from 'reactflow'

const CustomNode = React.memo(({ data }: NodeProps) => {
  return (
    <div className="px-4 py-2 shadow-md rounded-md bg-white border-2 border-stone-400">
      <Handle type="target" position={Position.Top} className="w-3 h-3" />
      <div className="text-sm font-medium">{data.label}</div>
      <Handle type="source" position={Position.Bottom} className="w-3 h-3" />
    </div>
  )
})

CustomNode.displayName = 'CustomNode'

export default CustomNode
