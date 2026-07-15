import { HashRouter, Routes, Route, Navigate } from 'react-router-dom'
import LandingPage from './LandingPage'
import DashboardPage from './DashboardPage'

export default function App() {
  return (
    <HashRouter>
      <Routes>
        <Route path="/" element={<LandingPage />} />
        <Route path="/dashboard" element={<DashboardPage />} />
        <Route path="*" element={<Navigate to="/" replace />} />
      </Routes>
    </HashRouter>
  )
}
