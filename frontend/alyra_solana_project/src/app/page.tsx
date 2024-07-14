import { AppHeader } from "@/components/AppHeader/appheader";
import { HomeTable } from "@/screens/Homescreen/hometable/hometable";

export default function Home() {
	return (
		<main className="min-h-screen p-24 bg-blue-600">
			<AppHeader />
			<HomeTable/>
		</main>
	);
}
