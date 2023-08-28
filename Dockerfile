FROM httpd:alpine

RUN mkdir -p /smrs/conf /smrs/htdocs /smrs/data
RUN chown -R daemon:daemon /smrs/data
RUN echo "Include /smrs/conf/smrs.conf" >> /usr/local/apache2/conf/httpd.conf

EXPOSE 8000
