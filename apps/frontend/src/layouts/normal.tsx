import { Outlet } from 'react-router';
import { Nav } from '../components/layout/nav';

export const Component = () => {
  return (
    <div>
      <Nav />
      <Outlet />
    </div>
  );
};
