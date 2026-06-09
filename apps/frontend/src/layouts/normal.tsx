import { Outlet } from 'react-router';
import { Nav } from '../components/layout/nav';

export const Component = () => {
  return (
    <div className="min-h-screen flex flex-col">
      <Nav />
      <Outlet />
    </div>
  );
};
