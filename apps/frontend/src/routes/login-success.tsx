import { redirect, type LoaderFunction } from 'react-router';

export const loader: LoaderFunction = ({ url }) => {
  const token = url.searchParams.get('token');

  if (token) localStorage.setItem('kanade.apikey', token);
  return redirect('/');
};
