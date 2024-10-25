--
-- PostgreSQL database dump
--

-- Dumped from database version 16.4
-- Dumped by pg_dump version 16.4

SET statement_timeout = 0;
SET lock_timeout = 0;
SET idle_in_transaction_session_timeout = 0;
SET client_encoding = 'UTF8';
SET standard_conforming_strings = on;
SELECT pg_catalog.set_config('search_path', '', false);
SET check_function_bodies = false;
SET xmloption = content;
SET client_min_messages = warning;
SET row_security = off;

--
-- Name: _sqlx_test; Type: SCHEMA; Schema: -; Owner: beeps
--

CREATE SCHEMA _sqlx_test;


ALTER SCHEMA _sqlx_test OWNER TO beeps;

--
-- Name: database_ids; Type: SEQUENCE; Schema: _sqlx_test; Owner: beeps
--

CREATE SEQUENCE _sqlx_test.database_ids
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE _sqlx_test.database_ids OWNER TO beeps;

SET default_tablespace = '';

SET default_table_access_method = heap;

--
-- Name: databases; Type: TABLE; Schema: _sqlx_test; Owner: beeps
--

CREATE TABLE _sqlx_test.databases (
    db_name text NOT NULL,
    test_path text NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);


ALTER TABLE _sqlx_test.databases OWNER TO beeps;

--
-- Name: _sqlx_migrations; Type: TABLE; Schema: public; Owner: beeps
--

CREATE TABLE public._sqlx_migrations (
    version bigint NOT NULL,
    description text NOT NULL,
    installed_on timestamp with time zone DEFAULT now() NOT NULL,
    success boolean NOT NULL,
    checksum bytea NOT NULL,
    execution_time bigint NOT NULL
);


ALTER TABLE public._sqlx_migrations OWNER TO beeps;

--
-- Name: accounts; Type: TABLE; Schema: public; Owner: beeps
--

CREATE TABLE public.accounts (
    id integer NOT NULL
);


ALTER TABLE public.accounts OWNER TO beeps;

--
-- Name: accounts_id_seq; Type: SEQUENCE; Schema: public; Owner: beeps
--

CREATE SEQUENCE public.accounts_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.accounts_id_seq OWNER TO beeps;

--
-- Name: accounts_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: beeps
--

ALTER SEQUENCE public.accounts_id_seq OWNED BY public.accounts.id;


--
-- Name: devices; Type: TABLE; Schema: public; Owner: beeps
--

CREATE TABLE public.devices (
    id integer NOT NULL,
    document_id bigint NOT NULL,
    name text NOT NULL,
    node_id bigint NOT NULL
);


ALTER TABLE public.devices OWNER TO beeps;

--
-- Name: devices_id_seq; Type: SEQUENCE; Schema: public; Owner: beeps
--

CREATE SEQUENCE public.devices_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.devices_id_seq OWNER TO beeps;

--
-- Name: devices_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: beeps
--

ALTER SEQUENCE public.devices_id_seq OWNED BY public.devices.id;


--
-- Name: documents; Type: TABLE; Schema: public; Owner: beeps
--

CREATE TABLE public.documents (
    id integer NOT NULL,
    account_id bigint NOT NULL
);


ALTER TABLE public.documents OWNER TO beeps;

--
-- Name: documents_id_seq; Type: SEQUENCE; Schema: public; Owner: beeps
--

CREATE SEQUENCE public.documents_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.documents_id_seq OWNER TO beeps;

--
-- Name: documents_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: beeps
--

ALTER SEQUENCE public.documents_id_seq OWNED BY public.documents.id;


--
-- Name: operations; Type: TABLE; Schema: public; Owner: beeps
--

CREATE TABLE public.operations (
    id integer NOT NULL,
    document_id bigint NOT NULL,
    "timestamp" timestamp with time zone NOT NULL,
    counter bigint NOT NULL,
    node smallint NOT NULL,
    op jsonb NOT NULL
);


ALTER TABLE public.operations OWNER TO beeps;

--
-- Name: operations_id_seq; Type: SEQUENCE; Schema: public; Owner: beeps
--

CREATE SEQUENCE public.operations_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.operations_id_seq OWNER TO beeps;

--
-- Name: operations_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: beeps
--

ALTER SEQUENCE public.operations_id_seq OWNED BY public.operations.id;


--
-- Name: accounts id; Type: DEFAULT; Schema: public; Owner: beeps
--

ALTER TABLE ONLY public.accounts ALTER COLUMN id SET DEFAULT nextval('public.accounts_id_seq'::regclass);


--
-- Name: devices id; Type: DEFAULT; Schema: public; Owner: beeps
--

ALTER TABLE ONLY public.devices ALTER COLUMN id SET DEFAULT nextval('public.devices_id_seq'::regclass);


--
-- Name: documents id; Type: DEFAULT; Schema: public; Owner: beeps
--

ALTER TABLE ONLY public.documents ALTER COLUMN id SET DEFAULT nextval('public.documents_id_seq'::regclass);


--
-- Name: operations id; Type: DEFAULT; Schema: public; Owner: beeps
--

ALTER TABLE ONLY public.operations ALTER COLUMN id SET DEFAULT nextval('public.operations_id_seq'::regclass);


--
-- Name: databases databases_pkey; Type: CONSTRAINT; Schema: _sqlx_test; Owner: beeps
--

ALTER TABLE ONLY _sqlx_test.databases
    ADD CONSTRAINT databases_pkey PRIMARY KEY (db_name);


--
-- Name: _sqlx_migrations _sqlx_migrations_pkey; Type: CONSTRAINT; Schema: public; Owner: beeps
--

ALTER TABLE ONLY public._sqlx_migrations
    ADD CONSTRAINT _sqlx_migrations_pkey PRIMARY KEY (version);


--
-- Name: accounts accounts_pkey; Type: CONSTRAINT; Schema: public; Owner: beeps
--

ALTER TABLE ONLY public.accounts
    ADD CONSTRAINT accounts_pkey PRIMARY KEY (id);


--
-- Name: devices devices_document_id_node_id_key; Type: CONSTRAINT; Schema: public; Owner: beeps
--

ALTER TABLE ONLY public.devices
    ADD CONSTRAINT devices_document_id_node_id_key UNIQUE (document_id, node_id);


--
-- Name: devices devices_pkey; Type: CONSTRAINT; Schema: public; Owner: beeps
--

ALTER TABLE ONLY public.devices
    ADD CONSTRAINT devices_pkey PRIMARY KEY (id);


--
-- Name: documents documents_pkey; Type: CONSTRAINT; Schema: public; Owner: beeps
--

ALTER TABLE ONLY public.documents
    ADD CONSTRAINT documents_pkey PRIMARY KEY (id);


--
-- Name: operations operations_pkey; Type: CONSTRAINT; Schema: public; Owner: beeps
--

ALTER TABLE ONLY public.operations
    ADD CONSTRAINT operations_pkey PRIMARY KEY (id);


--
-- Name: databases_created_at; Type: INDEX; Schema: _sqlx_test; Owner: beeps
--

CREATE INDEX databases_created_at ON _sqlx_test.databases USING btree (created_at);


--
-- Name: idx_document_node_timestamp_counter_desc; Type: INDEX; Schema: public; Owner: beeps
--

CREATE INDEX idx_document_node_timestamp_counter_desc ON public.operations USING btree (document_id, node, "timestamp" DESC, counter DESC);


--
-- Name: devices devices_document_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: beeps
--

ALTER TABLE ONLY public.devices
    ADD CONSTRAINT devices_document_id_fkey FOREIGN KEY (document_id) REFERENCES public.documents(id);


--
-- Name: documents documents_account_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: beeps
--

ALTER TABLE ONLY public.documents
    ADD CONSTRAINT documents_account_id_fkey FOREIGN KEY (account_id) REFERENCES public.accounts(id);


--
-- Name: operations operations_document_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: beeps
--

ALTER TABLE ONLY public.operations
    ADD CONSTRAINT operations_document_id_fkey FOREIGN KEY (document_id) REFERENCES public.documents(id);


--
-- PostgreSQL database dump complete
--

