"use client"

import type { ColumnDef } from "@tanstack/react-table"
import type { Thing } from "./types"

export const columns: ColumnDef<Thing>[] = [
  {
    accessorKey: "pool",
    header: "Pool",
  },
  {
    accessorKey: "description",
    header: "Description",
  },
  {
    accessorKey: "yield",
    header: "Yield",
  },
]
