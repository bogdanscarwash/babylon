import time
import logging
logger = logging.getLogger(__name__)

def retry_on_exception(max_retries=3, delay=1, exceptions=(Exception,)):
    def decorator(func):
        def wrapper(*args, **kwargs):
            retries = 0
            while retries < max_retries:
                try:
                    return func(*args, **kwargs)
                except exceptions as e:
                    retries += 1
                    logger.warning(f"Transient error in {func.__name__}: {e}. Retrying {retries}/{max_retries}...")
                    time.sleep(delay)
            logger.error(f"Failed after {max_retries} retries in {func.__name__}")
            raise
        return wrapper
    return decorator
