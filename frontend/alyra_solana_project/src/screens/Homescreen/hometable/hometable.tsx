import { columns } from "./columns";
import { DataTable } from "./data-table";
import { SAMPLE_DATA } from "./sample-data";

// export function getData(): Promise<Payment[]> {
// Fetch data from your API here.
//   return [
//     {
//       id: "728ed52f",
//       amount: 100,
//       status: "pending",
//       email: "m@example.com",
//     },
// // ...
//   ]
// }

export function HomeTable() {
	//   const data = await getData()

	return (
		<div className="container mx-auto py-10">
			<DataTable columns={columns} data={SAMPLE_DATA} />
		</div>
	);
}
