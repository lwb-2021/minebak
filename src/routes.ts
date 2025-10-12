import AboutView from "./components/AboutView.vue";
import HomeView from "./components/HomeView.vue";
import SettingsView from "./components/SettingsView.vue";

export const routes = [
  { path: '/', component: HomeView },
  { path: "/settings", component: SettingsView },
  { path: '/about', component: AboutView },
];
