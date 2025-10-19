import AboutView from "./views/AboutView.vue";
import HomeView from "./views/HomeView.vue";
import SettingsView from "./views/SettingsView.vue";
import SavesView from "./views/SavesView.vue";

export const routes = [
  { path: '/', component: HomeView },
  { path: "/settings", component: SettingsView },
  { path: "/saves", component: SavesView },
  { path: '/about', component: AboutView },
];
