macro_rules! infallible {
    ($t:ty) => {impl warp::Filter<Extract = ($t,), Error = std::convert::Infallible> + Clone + Send + Sync + 'static}
}

macro_rules! rejection {
    ($t:ty) => {impl warp::Filter<Extract = ($t,), Error = warp::Rejection> + Clone + Send + Sync + 'static}
}

macro_rules! reply {
    () => {impl warp::Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone + Send}
}
